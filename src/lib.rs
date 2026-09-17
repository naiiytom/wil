use std::{
    collections::HashSet,
    error::Error,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;

pub type RenderResult<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    #[serde(default)]
    title: String,
    #[serde(default)]
    canvas: Canvas,
    #[serde(default)]
    bounds: Option<[f64; 4]>,
    background: String,
    layers: Vec<Layer>,
    #[serde(default)]
    labels: Vec<Label>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Canvas {
    width: u32,
    height: u32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Layer {
    id: String,
    kind: String,
    #[serde(default)]
    fill: String,
    #[serde(default)]
    stroke: String,
    #[serde(default)]
    stroke_width: f32,
    #[serde(default)]
    lift: f32,
    #[serde(default)]
    geojson: Option<String>,
    #[serde(default)]
    points: Vec<[f32; 2]>,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    text: String,
    x: f32,
    y: f32,
    #[serde(default)]
    scene: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sources {
    sources: Vec<SourceReference>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceReference {
    id: String,
    title: String,
    url: String,
    license: String,
    retrieved: String,
    #[serde(default)]
    assume_crs: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequencePack {
    title: String,
    canvas: Canvas,
    bounds: [f64; 4],
    background: String,
    fps: u32,
    duration_seconds: f64,
    layers: Vec<Layer>,
    #[serde(default)]
    labels: Vec<Label>,
    scenes: Vec<SequenceScene>,
    #[serde(default)]
    animations: Vec<LayerAnimation>,
    #[serde(default)]
    label_animations: Vec<LabelAnimation>,
    #[serde(default)]
    map_transition: Option<MapTransition>,
    #[serde(default)]
    assume_crs: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceScene {
    id: String,
    start: f64,
    end: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerAnimation {
    layer: String,
    #[serde(default)]
    opacity: Vec<NumberKeyframe>,
    #[serde(default)]
    translate: TransformKeyframes,
    #[serde(default)]
    scale: Vec<NumberKeyframe>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelAnimation {
    label: String,
    #[serde(default)]
    opacity: Vec<NumberKeyframe>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberKeyframe {
    pub at: f64,
    pub value: f64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformKeyframes {
    #[serde(default)]
    x: Vec<NumberKeyframe>,
    #[serde(default)]
    y: Vec<NumberKeyframe>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapTransition {
    #[serde(default)]
    translate: TransformKeyframes,
    #[serde(default)]
    scale: Vec<NumberKeyframe>,
}

struct FrameScene<'a> {
    canvas: &'a Canvas,
    background: &'a str,
    layers: Vec<FrameLayer<'a>>,
    labels: Vec<FrameLabel<'a>>,
    map_transform: FrameTransform,
}

struct FrameLayer<'a> {
    layer: &'a Layer,
    transform: FrameTransform,
}

struct FrameLabel<'a> {
    label: &'a Label,
    opacity: f64,
}

#[derive(Clone, Copy)]
struct FrameTransform {
    opacity: f64,
    x: f64,
    y: f64,
    scale: f64,
}

impl FrameTransform {
    const IDENTITY: Self = Self {
        opacity: 1.0,
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum GeoJson {
    Feature { geometry: Box<GeoJson> },
    LineString { coordinates: Vec<[f64; 2]> },
    Polygon { coordinates: Vec<Vec<[f64; 2]>> },
}

pub fn render_source_pack(pack: impl AsRef<Path>, output: impl AsRef<Path>) -> RenderResult<()> {
    let pack = pack.as_ref();
    let mut scene: Scene = yaml_serde::from_str(&fs::read_to_string(pack.join("scene.yaml"))?)?;
    let sources: Sources = yaml_serde::from_str(&fs::read_to_string(pack.join("sources.yaml"))?)?;
    resolve_geojson(&mut scene, pack)?;
    validate(&scene, &sources)?;

    fs::write(output, render_png(static_frame_scene(&scene))?)?;
    Ok(())
}

pub fn render_sequence_frame(pack: impl AsRef<Path>, frame: usize) -> RenderResult<Vec<u8>> {
    let sequence = validate_sequence_pack(pack)?;
    render_sequence_frame_from_sequence(&sequence, frame)
}

pub fn render_sequence_pack(pack: impl AsRef<Path>, output: impl AsRef<Path>) -> RenderResult<()> {
    let output = output.as_ref();
    if output.exists() {
        return Err("sequence render output already exists".into());
    }

    let sequence = validate_sequence_pack(pack)?;
    let frame_count = sequence_frame_count(&sequence);
    let temporary = temporary_output_directory(output)?;
    let result = (|| {
        for frame in 0..frame_count {
            fs::write(
                temporary.join(format!("frame-{frame:06}.png")),
                render_sequence_frame_from_sequence(&sequence, frame)?,
            )?;
        }
        fs::write(
            temporary.join("render-manifest.yaml"),
            render_manifest(&sequence, frame_count),
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error.into());
    }
    Ok(())
}

fn temporary_output_directory(output: &Path) -> RenderResult<PathBuf> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .ok_or("sequence render output needs a directory name")?
        .to_string_lossy();
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    for attempt in 0..100 {
        let temporary = parent.join(format!(
            ".{name}.rendering-{}-{nonce}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&temporary) {
            Ok(()) => return Ok(temporary),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err("could not create a unique sequence render directory".into())
}

fn render_manifest(sequence: &SequencePack, frame_count: usize) -> String {
    let mut manifest = format!(
        "title: {:?}\nfps: {}\nduration_seconds: {}\nframe_count: {frame_count}\nframe_pattern: frame-%06d.png\nscenes:\n",
        sequence.title, sequence.fps, sequence.duration_seconds
    );
    for scene in &sequence.scenes {
        manifest.push_str(&format!(
            "  - id: {:?}\n    start: {}\n    end: {}\n",
            scene.id, scene.start, scene.end
        ));
    }
    manifest.push_str(
        "attribution_card: \"Geographic features are source-backed; timing is illustrative.\"\n",
    );
    manifest
}

fn render_sequence_frame_from_sequence(
    sequence: &SequencePack,
    frame: usize,
) -> RenderResult<Vec<u8>> {
    let frame_count = sequence_frame_count(sequence);
    if frame >= frame_count {
        return Err("sequence frame is outside the sequence duration".into());
    }
    render_png(frame_scene(
        sequence,
        frame as f64 / f64::from(sequence.fps),
    ))
}

fn sequence_frame_count(sequence: &SequencePack) -> usize {
    (sequence.duration_seconds * f64::from(sequence.fps)).round() as usize
}

fn render_png(scene: FrameScene<'_>) -> RenderResult<Vec<u8>> {
    let svg = compose_svg(&scene);
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(&svg, &options)?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(scene.canvas.width, scene.canvas.height)
        .ok_or("invalid canvas size")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}

pub fn validate_sequence_pack(pack: impl AsRef<Path>) -> RenderResult<SequencePack> {
    let pack = pack.as_ref();
    let mut sequence: SequencePack =
        yaml_serde::from_str(&fs::read_to_string(pack.join("sequence.yaml"))?)?;
    let sources = yaml_serde::from_str(&fs::read_to_string(pack.join("sources.yaml"))?)?;
    resolve_geojson_layers(
        &mut sequence.layers,
        &sequence.canvas,
        Some(sequence.bounds),
        pack,
    )?;
    validate_sequence(&sequence, &sources)?;
    Ok(sequence)
}

fn static_frame_scene(scene: &Scene) -> FrameScene<'_> {
    FrameScene {
        canvas: &scene.canvas,
        background: &scene.background,
        layers: scene
            .layers
            .iter()
            .map(|layer| FrameLayer {
                layer,
                transform: FrameTransform::IDENTITY,
            })
            .collect(),
        labels: scene
            .labels
            .iter()
            .map(|label| FrameLabel {
                label,
                opacity: 1.0,
            })
            .collect(),
        map_transform: FrameTransform::IDENTITY,
    }
}

fn frame_scene(sequence: &SequencePack, time: f64) -> FrameScene<'_> {
    let map_transform = sequence
        .map_transition
        .as_ref()
        .map(|transition| FrameTransform {
            opacity: 1.0,
            x: interpolate(&transition.translate.x, time, 0.0),
            y: interpolate(&transition.translate.y, time, 0.0),
            scale: interpolate(&transition.scale, time, 1.0),
        })
        .unwrap_or(FrameTransform::IDENTITY);
    let layers = sequence
        .layers
        .iter()
        .map(|layer| {
            let transform = sequence
                .animations
                .iter()
                .find(|animation| animation.layer == layer.id)
                .map(|animation| FrameTransform {
                    opacity: interpolate(&animation.opacity, time, 1.0),
                    x: interpolate(&animation.translate.x, time, 0.0),
                    y: interpolate(&animation.translate.y, time, 0.0),
                    scale: interpolate(&animation.scale, time, 1.0),
                })
                .unwrap_or(FrameTransform::IDENTITY);
            FrameLayer { layer, transform }
        })
        .collect();
    let active_scene = sequence
        .scenes
        .iter()
        .find(|scene| time >= scene.start && time < scene.end)
        .map(|scene| scene.id.as_str());
    FrameScene {
        canvas: &sequence.canvas,
        background: &sequence.background,
        layers,
        labels: sequence
            .labels
            .iter()
            .filter(|label| {
                label
                    .scene
                    .as_deref()
                    .is_none_or(|scene| Some(scene) == active_scene)
            })
            .map(|label| {
                let opacity = sequence
                    .label_animations
                    .iter()
                    .find(|a| a.label == label.text)
                    .map(|a| interpolate(&a.opacity, time, 1.0))
                    .unwrap_or(1.0);
                FrameLabel { label, opacity }
            })
            .collect(),
        map_transform,
    }
}

pub fn interpolate(keyframes: &[NumberKeyframe], time: f64, default: f64) -> f64 {
    let Some(first) = keyframes.first() else {
        return default;
    };
    if time <= first.at {
        return first.value;
    }
    for pair in keyframes.windows(2) {
        if time <= pair[1].at {
            let fraction = (time - pair[0].at) / (pair[1].at - pair[0].at);
            return pair[0].value + (pair[1].value - pair[0].value) * fraction;
        }
    }
    keyframes.last().unwrap().value
}

fn resolve_geojson(scene: &mut Scene, pack: &Path) -> RenderResult<()> {
    resolve_geojson_layers(&mut scene.layers, &scene.canvas, scene.bounds, pack)
}

fn resolve_geojson_layers(
    layers: &mut [Layer],
    canvas: &Canvas,
    bounds: Option<[f64; 4]>,
    pack: &Path,
) -> RenderResult<()> {
    let Some([west, south, east, north]) = bounds else {
        if layers.iter().any(|layer| layer.geojson.is_some()) {
            return Err("Scene bounds are required for GeoJSON layers".into());
        }
        return Ok(());
    };
    if east <= west || north <= south {
        return Err("Scene bounds must be west, south, east, north".into());
    }
    for layer in layers {
        let Some(path) = &layer.geojson else {
            continue;
        };
        let geojson: GeoJson = yaml_serde::from_str(&fs::read_to_string(pack.join(path))?)?;
        let (kind, coordinates) = geometry_coordinates(geojson)?;
        if (layer.kind == "line" && kind != "LineString")
            || (layer.kind == "polygon" && kind != "Polygon")
        {
            return Err(format!("Layer '{}' does not match GeoJSON {kind}", layer.id).into());
        }
        layer.points = coordinates
            .into_iter()
            .map(|[longitude, latitude]| {
                [
                    ((longitude - west) / (east - west) * canvas.width as f64) as f32,
                    ((north - latitude) / (north - south) * canvas.height as f64) as f32,
                ]
            })
            .collect();
    }
    Ok(())
}

fn geometry_coordinates(geojson: GeoJson) -> RenderResult<(&'static str, Vec<[f64; 2]>)> {
    match geojson {
        GeoJson::Feature { geometry } => geometry_coordinates(*geometry),
        GeoJson::LineString { coordinates } => Ok(("LineString", coordinates)),
        GeoJson::Polygon { coordinates } => coordinates
            .into_iter()
            .next()
            .map(|ring| ("Polygon", ring))
            .ok_or_else(|| "GeoJSON Polygon needs a linear ring".into()),
    }
}

fn validate(scene: &Scene, sources: &Sources) -> RenderResult<()> {
    validate_source_metadata(sources)?;
    let source_ids: HashSet<&str> = sources
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    for layer in &scene.layers {
        if !matches!(layer.kind.as_str(), "polygon" | "line") {
            return Err(format!(
                "Layer '{}' has unknown layer kind '{}'",
                layer.id, layer.kind
            )
            .into());
        }
        if layer.points.len() < if layer.kind == "polygon" { 3 } else { 2 } {
            return Err(format!("Layer '{}' has too few points", layer.id).into());
        }
        validate_references(
            &layer.sources,
            &source_ids,
            &format!("Layer '{}'", layer.id),
        )?;
    }
    for label in &scene.labels {
        validate_references(
            &label.sources,
            &source_ids,
            &format!("Label '{}'", label.text),
        )?;
    }
    Ok(())
}

fn validate_sequence(sequence: &SequencePack, sources: &Sources) -> RenderResult<()> {
    if sequence.fps == 0
        || !sequence.duration_seconds.is_finite()
        || sequence.duration_seconds <= 0.0
    {
        return Err("sequence fps and duration_seconds must be positive".into());
    }
    let frames = sequence.duration_seconds * f64::from(sequence.fps);
    if (frames - frames.round()).abs() > 1e-9 {
        return Err("duration_seconds * fps must be a whole number of frames".into());
    }
    validate_crs(sequence.assume_crs.as_deref())?;
    validate_source_metadata(sources)?;

    let layer_ids: HashSet<&str> = sequence
        .layers
        .iter()
        .map(|layer| layer.id.as_str())
        .collect();
    if layer_ids.len() != sequence.layers.len() {
        return Err("sequence layer IDs must be unique".into());
    }
    let source_ids: HashSet<&str> = sources
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    for layer in &sequence.layers {
        if !matches!(layer.kind.as_str(), "polygon" | "line") {
            return Err(format!(
                "Layer '{}' has unknown layer kind '{}'",
                layer.id, layer.kind
            )
            .into());
        }
        if layer.points.len() < if layer.kind == "polygon" { 3 } else { 2 } {
            return Err(format!("Layer '{}' has too few points", layer.id).into());
        }
        validate_references(
            &layer.sources,
            &source_ids,
            &format!("Layer '{}'", layer.id),
        )?;
    }
    for label in &sequence.labels {
        validate_references(
            &label.sources,
            &source_ids,
            &format!("Label '{}'", label.text),
        )?;
    }

    let mut scene_ids = HashSet::new();
    let mut end = 0.0;
    for scene in &sequence.scenes {
        if !scene_ids.insert(scene.id.as_str()) {
            return Err("sequence Scene IDs must be unique".into());
        }
        if !scene.start.is_finite()
            || !scene.end.is_finite()
            || scene.start != end
            || scene.end <= scene.start
        {
            return Err("sequence Scenes must be ordered contiguous [start, end) ranges".into());
        }
        end = scene.end;
    }
    if end != sequence.duration_seconds {
        return Err("sequence Scenes must span zero through duration_seconds".into());
    }
    for label in &sequence.labels {
        if let Some(scene) = &label.scene {
            if !scene_ids.contains(scene.as_str()) {
                return Err(
                    format!("Label '{}' references unknown Scene '{scene}'", label.text).into(),
                );
            }
        }
    }

    let mut animation_layers = HashSet::new();
    for animation in &sequence.animations {
        if !layer_ids.contains(animation.layer.as_str()) {
            return Err(format!("animation references unknown layer '{}'", animation.layer).into());
        }
        if !animation_layers.insert(animation.layer.as_str()) {
            return Err("animation target layers must be unique".into());
        }
        validate_keyframes(&animation.opacity, sequence.duration_seconds, "opacity")?;
        validate_keyframes(
            &animation.translate.x,
            sequence.duration_seconds,
            "translate.x",
        )?;
        validate_keyframes(
            &animation.translate.y,
            sequence.duration_seconds,
            "translate.y",
        )?;
        validate_keyframes(&animation.scale, sequence.duration_seconds, "scale")?;
    }
    if let Some(transition) = &sequence.map_transition {
        validate_keyframes(
            &transition.translate.x,
            sequence.duration_seconds,
            "map transition translate.x",
        )?;
        validate_keyframes(
            &transition.translate.y,
            sequence.duration_seconds,
            "map transition translate.y",
        )?;
        validate_keyframes(
            &transition.scale,
            sequence.duration_seconds,
            "map transition scale",
        )?;
    }
    let label_texts: HashSet<&str> = sequence.labels.iter().map(|l| l.text.as_str()).collect();
    let mut animation_labels = HashSet::new();
    for animation in &sequence.label_animations {
        if !label_texts.contains(animation.label.as_str()) {
            return Err(format!(
                "label_animation references unknown label '{}'",
                animation.label
            )
            .into());
        }
        if !animation_labels.insert(animation.label.as_str()) {
            return Err("label_animation target labels must be unique".into());
        }
        validate_keyframes(
            &animation.opacity,
            sequence.duration_seconds,
            "label opacity",
        )?;
    }
    Ok(())
}

fn validate_source_metadata(sources: &Sources) -> RenderResult<()> {
    for source in &sources.sources {
        if source.title.is_empty()
            || source.url.is_empty()
            || source.license.is_empty()
            || source.retrieved.is_empty()
        {
            return Err(
                "each source reference needs title, URL, license, and retrieval date".into(),
            );
        }
        validate_crs(source.assume_crs.as_deref())?;
    }
    Ok(())
}

fn validate_crs(assume_crs: Option<&str>) -> RenderResult<()> {
    match assume_crs {
        None => Ok(()),
        Some("EPSG:4326") => Ok(()),
        Some(value) => Err(format!(
            "assume_crs '{value}' is not supported; only EPSG:4326 is accepted until projection conversion is implemented"
        )
        .into()),
    }
}

fn validate_keyframes(
    keyframes: &[NumberKeyframe],
    duration_seconds: f64,
    property: &str,
) -> RenderResult<()> {
    let mut previous = None;
    for keyframe in keyframes {
        if !keyframe.at.is_finite()
            || !keyframe.value.is_finite()
            || keyframe.at < 0.0
            || keyframe.at > duration_seconds
        {
            return Err(
                format!("{property} keyframes must be within the sequence duration").into(),
            );
        }
        if previous.is_some_and(|at| keyframe.at <= at) {
            return Err(format!("{property} keyframes must be monotonic").into());
        }
        previous = Some(keyframe.at);
    }
    Ok(())
}

fn validate_references(
    references: &[String],
    known: &HashSet<&str>,
    owner: &str,
) -> RenderResult<()> {
    if references.is_empty() {
        return Err(format!("{owner} needs at least one source reference").into());
    }
    if let Some(id) = references.iter().find(|id| !known.contains(id.as_str())) {
        return Err(format!("{owner} references unknown source '{id}'").into());
    }
    Ok(())
}

fn compose_svg(scene: &FrameScene<'_>) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><defs><pattern id="paper-grain" width="14" height="14" patternUnits="userSpaceOnUse"><path d="M0 2L14 0M0 10L14 8" stroke="#ffffff" stroke-opacity="0.18" stroke-width="1"/></pattern></defs><rect width="100%" height="100%" fill="{}"/><rect width="100%" height="100%" fill="url(#paper-grain)"/>"##,
        scene.canvas.width,
        scene.canvas.height,
        scene.canvas.width,
        scene.canvas.height,
        scene.background
    );
    svg.push_str(&transform_group(scene.map_transform));
    for frame_layer in &scene.layers {
        let layer = frame_layer.layer;
        svg.push_str(&format!(
            r#"<g opacity="{}" transform="translate({} {}) scale({})">"#,
            frame_layer.transform.opacity,
            frame_layer.transform.x,
            frame_layer.transform.y,
            frame_layer.transform.scale
        ));
        let points = layer
            .points
            .iter()
            .map(|[x, y]| format!("{x},{y}"))
            .collect::<Vec<_>>()
            .join(" ");
        let fill = if layer.fill.is_empty() {
            "none"
        } else {
            &layer.fill
        };
        let stroke = if layer.stroke.is_empty() {
            "#f5eedb"
        } else {
            &layer.stroke
        };
        let stroke_width = if layer.stroke_width == 0.0 {
            5.0
        } else {
            layer.stroke_width
        };
        let lift = layer.lift.max(0.0);
        match layer.kind.as_str() {
            "polygon" => {
                if lift > 0.0 {
                    svg.push_str(&format!(r##"<polygon points="{}" fill="#514d42" fill-opacity="0.34" transform="translate(0 {})"/>"##, points, lift));
                }
                svg.push_str(&format!(r#"<polygon id="{}" points="{}" fill="{}" stroke="{}" stroke-width="{}" stroke-linejoin="round"/>"#, escape(&layer.id), points, fill, stroke, stroke_width));
                svg.push_str(&format!(
                    r#"<polygon points="{}" fill="url(#paper-grain)" fill-opacity="0.36"/>"#,
                    points
                ));
            }
            "line" => {
                if lift > 0.0 {
                    svg.push_str(&format!(r##"<polyline points="{}" fill="none" stroke="#514d42" stroke-opacity="0.34" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round" transform="translate(0 {})"/>"##, points, stroke_width, lift));
                }
                svg.push_str(&format!(r#"<polyline id="{}" points="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#, escape(&layer.id), points, stroke, stroke_width));
            }
            _ => {}
        }
        svg.push_str("</g>");
    }
    for frame_label in &scene.labels {
        let label = frame_label.label;
        let opacity_attr = if frame_label.opacity < 1.0 {
            format!(" opacity=\"{:.3}\"", frame_label.opacity)
        } else {
            String::new()
        };
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#252525" font-family="sans-serif" font-size="16" font-weight="700"{}>{}</text>"##, label.x, label.y, opacity_attr, escape(&label.text)));
    }
    svg.push_str("</g></svg>");
    svg
}

fn transform_group(transform: FrameTransform) -> String {
    format!(
        r#"<g transform="translate({} {}) scale({})">"#,
        transform.x, transform.y, transform.scale
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
