use std::{collections::HashSet, error::Error, fs, path::Path};

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
pub struct NumberKeyframe {
    at: f64,
    value: f64,
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
    pixmap.save_png(output)?;
    Ok(())
}

pub fn validate_sequence_pack(pack: impl AsRef<Path>) -> RenderResult<SequencePack> {
    let pack = pack.as_ref();
    let sequence = yaml_serde::from_str(&fs::read_to_string(pack.join("sequence.yaml"))?)?;
    let sources = yaml_serde::from_str(&fs::read_to_string(pack.join("sources.yaml"))?)?;
    validate_sequence(&sequence, &sources)?;
    Ok(sequence)
}

fn resolve_geojson(scene: &mut Scene, pack: &Path) -> RenderResult<()> {
    let Some([west, south, east, north]) = scene.bounds else {
        if scene.layers.iter().any(|layer| layer.geojson.is_some()) {
            return Err("Scene bounds are required for GeoJSON layers".into());
        }
        return Ok(());
    };
    if east <= west || north <= south {
        return Err("Scene bounds must be west, south, east, north".into());
    }
    for layer in &mut scene.layers {
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
                    ((longitude - west) / (east - west) * scene.canvas.width as f64) as f32,
                    ((north - latitude) / (north - south) * scene.canvas.height as f64) as f32,
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

    for animation in &sequence.animations {
        if !layer_ids.contains(animation.layer.as_str()) {
            return Err(format!("animation references unknown layer '{}'", animation.layer).into());
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
    if assume_crs.is_some_and(|value| !value.starts_with("EPSG:")) {
        return Err("assume_crs must begin with EPSG:".into());
    }
    Ok(())
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

fn compose_svg(scene: &Scene) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><defs><pattern id="paper-grain" width="14" height="14" patternUnits="userSpaceOnUse"><path d="M0 2L14 0M0 10L14 8" stroke="#ffffff" stroke-opacity="0.18" stroke-width="1"/></pattern></defs><rect width="100%" height="100%" fill="{}"/><rect width="100%" height="100%" fill="url(#paper-grain)"/>"##,
        scene.canvas.width,
        scene.canvas.height,
        scene.canvas.width,
        scene.canvas.height,
        scene.background
    );
    for layer in &scene.layers {
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
    }
    for label in &scene.labels {
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#252525" font-family="sans-serif" font-size="16" font-weight="700">{}</text>"##, label.x, label.y, escape(&label.text)));
    }
    svg.push_str("</svg>");
    svg
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
