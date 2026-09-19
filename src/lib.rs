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
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    pub sources: Vec<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReference {
    pub id: String,
    pub title: String,
    pub url: String,
    pub license: String,
    pub retrieved: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assume_crs: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequencePack {
    title: String,
    canvas: Canvas,
    bounds: [f64; 4],
    background: String,
    pub fps: u32,
    pub duration_seconds: f64,
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

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodePack {
    pub title: String,
    pub episode_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_runtime_seconds: Option<f64>,
    pub sequences: Vec<EpisodeSequenceEntry>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeSequenceEntry {
    pub id: String,
    pub pack: String,
    pub section: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCategoryGroup {
    pub category: String,
    pub sources: Vec<SourceReference>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeManifestSequence {
    pub id: String,
    pub pack: String,
    pub section: String,
    pub directory: String,
    pub fps: u32,
    pub duration_seconds: f64,
    pub frame_count: usize,
    pub frame_pattern: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeManifest {
    pub title: String,
    pub episode_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_runtime_seconds: Option<f64>,
    pub rendered_duration_seconds: f64,
    pub total_frame_count: usize,
    pub sequences: Vec<EpisodeManifestSequence>,
    pub attribution_card: String,
    pub sources: Vec<SourceCategoryGroup>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceScene {
    pub id: String,
    pub start: f64,
    pub end: f64,
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
    render_sequence_pack_with_jobs(pack, output, None)
}

pub fn render_sequence_pack_with_jobs(
    pack: impl AsRef<Path>,
    output: impl AsRef<Path>,
    jobs: Option<usize>,
) -> RenderResult<()> {
    let output = output.as_ref();
    if output.exists() {
        return Err("sequence render output already exists".into());
    }

    let sequence = validate_sequence_pack(pack)?;
    let frame_count = sequence_frame_count(&sequence);
    let temporary = temporary_output_directory(output)?;
    let fontdb = system_fontdb();

    let render_frames = || {
        use rayon::prelude::*;
        (0..frame_count).into_par_iter().try_for_each(|frame| {
            let frame_bytes = render_sequence_frame_from_sequence_with_db(&sequence, frame, &fontdb)
                .map_err(|e| e.to_string())?;
            fs::write(
                temporary.join(format!("frame-{frame:06}.png")),
                frame_bytes,
            )
            .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        })
    };

    let result: Result<(), Box<dyn Error>> = match jobs {
        Some(threads) => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()?;
            pool.install(render_frames).map_err(|e| e.into())
        }
        None => render_frames().map_err(|e| e.into()),
    };

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }

    if let Err(error) = fs::write(
        temporary.join("render-manifest.yaml"),
        render_manifest(&sequence, frame_count)?,
    ) {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error.into());
    }

    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error.into());
    }
    Ok(())
}

pub fn load_sources(pack: impl AsRef<Path>) -> RenderResult<Sources> {
    let pack = pack.as_ref();
    let sources_file = pack.join("sources.yaml");
    if !sources_file.exists() {
        return Err(format!("missing sources.yaml in {}", pack.display()).into());
    }
    let content = fs::read_to_string(sources_file)?;
    let sources: Sources = yaml_serde::from_str(&content)?;
    Ok(sources)
}

pub fn validate_episode_pack(pack: impl AsRef<Path>) -> RenderResult<EpisodePack> {
    let pack = pack.as_ref();
    let episode_file = pack.join("episode.yaml");
    if !episode_file.exists() {
        return Err(format!("missing episode.yaml in {}", pack.display()).into());
    }
    let content = fs::read_to_string(&episode_file)?;
    let episode: EpisodePack = yaml_serde::from_str(&content)?;

    if episode.title.trim().is_empty() {
        return Err("episode title cannot be empty".into());
    }
    if episode.episode_id.trim().is_empty() {
        return Err("episode_id cannot be empty".into());
    }
    if episode.sequences.is_empty() {
        return Err("episode must declare at least one sequence".into());
    }

    let mut seen_seq_ids = std::collections::HashSet::new();
    let mut all_sources: std::collections::HashMap<String, (SourceReference, String)> =
        std::collections::HashMap::new();

    let shared_sources_file = pack.join("_shared").join("sources.yaml");
    if shared_sources_file.exists() {
        let shared_content = fs::read_to_string(&shared_sources_file)?;
        let shared_sources: Sources = yaml_serde::from_str(&shared_content)?;
        for source in shared_sources.sources {
            all_sources.insert(source.id.clone(), (source, "_shared".to_string()));
        }
    }

    for entry in &episode.sequences {
        if entry.id.trim().is_empty() {
            return Err("sequence entry id cannot be empty".into());
        }
        if !seen_seq_ids.insert(&entry.id) {
            return Err(format!("duplicate sequence id '{}' in episode", entry.id).into());
        }
        let child_pack = pack.join(&entry.pack);
        if !child_pack.is_dir() {
            return Err(format!("sequence pack directory '{}' not found", child_pack.display()).into());
        }

        validate_sequence_pack(&child_pack)?;

        let child_sources = load_sources(&child_pack)?;
        for source in child_sources.sources {
            if let Some((existing, prev_pack)) = all_sources.get(&source.id) {
                if existing.title != source.title
                    || existing.url != source.url
                    || existing.license != source.license
                    || existing.assume_crs != source.assume_crs
                {
                    return Err(format!(
                        "conflicting source definition for '{}' between '{}' and '{}'",
                        source.id, prev_pack, entry.pack
                    ).into());
                }
            } else {
                all_sources.insert(source.id.clone(), (source, entry.pack.clone()));
            }
        }
    }

    Ok(episode)
}

pub fn render_episode_pack(
    pack: impl AsRef<Path>,
    output: impl AsRef<Path>,
    jobs: Option<usize>,
) -> RenderResult<()> {
    let pack = pack.as_ref();
    let output = output.as_ref();
    if output.exists() {
        return Err("episode render output already exists".into());
    }

    let episode = validate_episode_pack(pack)?;
    let temporary = temporary_output_directory(output)?;

    let result = (|| -> RenderResult<()> {
        let mut manifest_sequences = Vec::new();
        let mut total_frame_count = 0;
        let mut rendered_duration_seconds = 0.0;
        let mut all_unique_sources: std::collections::BTreeMap<String, SourceReference> =
            std::collections::BTreeMap::new();

        for entry in &episode.sequences {
            let child_pack_dir = pack.join(&entry.pack);
            let seq_output_dir = temporary.join(&entry.pack);

            render_sequence_pack_with_jobs(&child_pack_dir, &seq_output_dir, jobs)?;

            let child_seq = validate_sequence_pack(&child_pack_dir)?;
            let child_frame_count = sequence_frame_count(&child_seq);
            total_frame_count += child_frame_count;
            rendered_duration_seconds += child_seq.duration_seconds;

            let child_sources = load_sources(&child_pack_dir)?;
            for s in child_sources.sources {
                all_unique_sources.entry(s.id.clone()).or_insert(s);
            }

            manifest_sequences.push(EpisodeManifestSequence {
                id: entry.id.clone(),
                pack: entry.pack.clone(),
                section: entry.section.clone(),
                directory: entry.pack.clone(),
                fps: child_seq.fps,
                duration_seconds: child_seq.duration_seconds,
                frame_count: child_frame_count,
                frame_pattern: "frame-%06d.png".to_string(),
            });
        }

        if let Ok(shared_sources) = load_sources(pack.join("_shared")) {
            for s in shared_sources.sources {
                all_unique_sources.entry(s.id.clone()).or_insert(s);
            }
        }

        let mut grouped: std::collections::BTreeMap<String, Vec<SourceReference>> =
            std::collections::BTreeMap::new();
        for (_, source) in all_unique_sources {
            let cat = source_category(&source);
            grouped.entry(cat).or_default().push(source);
        }

        let standard_order = [
            "Geographic & Elevation Baselines",
            "Historical & Morphological Surveys",
            "Hydrological & Infrastructure Records",
        ];
        let mut source_groups = Vec::new();
        for cat in standard_order {
            if let Some(sources) = grouped.remove(cat) {
                source_groups.push(SourceCategoryGroup {
                    category: cat.to_string(),
                    sources,
                });
            }
        }
        for (category, sources) in grouped {
            source_groups.push(SourceCategoryGroup { category, sources });
        }

        let episode_manifest = EpisodeManifest {
            title: episode.title.clone(),
            episode_id: episode.episode_id.clone(),
            target_runtime_seconds: episode.target_runtime_seconds,
            rendered_duration_seconds,
            total_frame_count,
            sequences: manifest_sequences,
            attribution_card: "Geographic features are source-backed; timing is illustrative.".to_string(),
            sources: source_groups,
        };

        fs::write(
            temporary.join("episode-manifest.yaml"),
            yaml_serde::to_string(&episode_manifest)?,
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

fn source_category(source: &SourceReference) -> String {
    if let Some(cat) = &source.category {
        return cat.clone();
    }
    let id_lower = source.id.to_lowercase();
    if id_lower.contains("dem")
        || id_lower.contains("elevation")
        || id_lower.contains("osm")
        || id_lower.contains("hydro")
        || id_lower.contains("coastline")
    {
        "Geographic & Elevation Baselines".to_string()
    } else if id_lower.contains("rtsd")
        || id_lower.contains("rsd")
        || id_lower.contains("bradley")
        || id_lower.contains("ghsl")
        || id_lower.contains("historic")
        || id_lower.contains("ams")
        || id_lower.contains("loftus")
    {
        "Historical & Morphological Surveys".to_string()
    } else if id_lower.contains("bma")
        || id_lower.contains("jica")
        || id_lower.contains("wmo")
        || id_lower.contains("chula")
        || id_lower.contains("drainage")
        || id_lower.contains("flood")
        || id_lower.contains("groundwater")
        || id_lower.contains("subsidence")
    {
        "Hydrological & Infrastructure Records".to_string()
    } else {
        "General Sources".to_string()
    }
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

#[derive(serde::Serialize)]
struct RenderManifest<'a> {
    title: &'a str,
    fps: u32,
    duration_seconds: f64,
    frame_count: usize,
    frame_pattern: &'static str,
    scenes: &'a [SequenceScene],
    attribution_card: &'static str,
}

fn render_manifest(sequence: &SequencePack, frame_count: usize) -> RenderResult<String> {
    let manifest = RenderManifest {
        title: &sequence.title,
        fps: sequence.fps,
        duration_seconds: sequence.duration_seconds,
        frame_count,
        frame_pattern: "frame-%06d.png",
        scenes: &sequence.scenes,
        attribution_card: "Geographic features are source-backed; timing is illustrative.",
    };
    Ok(yaml_serde::to_string(&manifest)?)
}

fn system_fontdb() -> std::sync::Arc<resvg::usvg::fontdb::Database> {
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    std::sync::Arc::new(fontdb)
}

fn render_sequence_frame_from_sequence(
    sequence: &SequencePack,
    frame: usize,
) -> RenderResult<Vec<u8>> {
    render_sequence_frame_from_sequence_with_db(sequence, frame, &system_fontdb())
}

fn render_sequence_frame_from_sequence_with_db(
    sequence: &SequencePack,
    frame: usize,
    fontdb: &std::sync::Arc<resvg::usvg::fontdb::Database>,
) -> RenderResult<Vec<u8>> {
    let frame_count = sequence_frame_count(sequence);
    if frame >= frame_count {
        return Err("sequence frame is outside the sequence duration".into());
    }
    render_png_with_db(
        frame_scene(
            sequence,
            frame as f64 / f64::from(sequence.fps),
        ),
        fontdb,
    )
}

fn sequence_frame_count(sequence: &SequencePack) -> usize {
    (sequence.duration_seconds * f64::from(sequence.fps)).round() as usize
}

fn render_png(scene: FrameScene<'_>) -> RenderResult<Vec<u8>> {
    render_png_with_db(scene, &system_fontdb())
}

fn render_png_with_db(
    scene: FrameScene<'_>,
    fontdb: &std::sync::Arc<resvg::usvg::fontdb::Database>,
) -> RenderResult<Vec<u8>> {
    let svg = compose_svg(&scene);
    let options = resvg::usvg::Options {
        fontdb: std::sync::Arc::clone(fontdb),
        ..Default::default()
    };
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
        sequence.assume_crs.as_deref(),
        Some(&sources),
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
        .map(|transition| interpolate_spatial(&transition.translate, &transition.scale, 1.0, time))
        .unwrap_or(FrameTransform::IDENTITY);
    let layers = sequence
        .layers
        .iter()
        .map(|layer| {
            let transform = sequence
                .animations
                .iter()
                .find(|animation| animation.layer == layer.id)
                .map(|animation| {
                    interpolate_spatial(
                        &animation.translate,
                        &animation.scale,
                        interpolate(&animation.opacity, time, 1.0),
                        time,
                    )
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

fn interpolate_spatial(
    translate: &TransformKeyframes,
    scale: &[NumberKeyframe],
    opacity: f64,
    time: f64,
) -> FrameTransform {
    FrameTransform {
        opacity,
        x: interpolate(&translate.x, time, 0.0),
        y: interpolate(&translate.y, time, 0.0),
        scale: interpolate(scale, time, 1.0),
    }
}

fn resolve_geojson(scene: &mut Scene, pack: &Path) -> RenderResult<()> {
    resolve_geojson_layers(
        &mut scene.layers,
        &scene.canvas,
        scene.bounds,
        pack,
        None,
        None,
    )
}

fn resolve_geojson_layers(
    layers: &mut [Layer],
    canvas: &Canvas,
    bounds: Option<[f64; 4]>,
    pack: &Path,
    assume_crs: Option<&str>,
    sources: Option<&Sources>,
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
        let raw = fs::read_to_string(pack.join(path))?;
        let value: yaml_serde::Value = yaml_serde::from_str(&raw)?;
        if sources.is_some() {
            let has_crs = match &value {
                yaml_serde::Value::Mapping(map) => map.keys().any(|k| match k {
                    yaml_serde::Value::String(s) => s == "crs",
                    _ => false,
                }),
                _ => false,
            };
            if has_crs {
                validate_geojson_crs(&value, path)?;
            } else {
                let has_source_assume = sources.is_some_and(|s| {
                    layer.sources.iter().any(|source_id| {
                        s.sources
                            .iter()
                            .any(|src| src.id == *source_id && src.assume_crs.is_some())
                    })
                });
                if assume_crs.is_none() && !has_source_assume {
                    return Err(format!(
                        "Source input '{path}' lacks CRS metadata; declare an explicit 'assume_crs: EPSG:…' in the manifest"
                    )
                    .into());
                }
            }
        }
        let geojson: GeoJson = yaml_serde::from_str(&raw)?;
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

fn validate_geojson_crs(value: &yaml_serde::Value, path: &str) -> RenderResult<()> {
    if let yaml_serde::Value::Mapping(map) = value {
        for (k, v) in map {
            if let yaml_serde::Value::String(k_str) = k
                && k_str == "crs"
            {
                let crs_name = match v {
                    yaml_serde::Value::String(s) => Some(s.as_str()),
                    yaml_serde::Value::Mapping(crs_map) => {
                        crs_map.iter().find_map(|(ck, cv)| {
                            if let (
                                yaml_serde::Value::String(ck_str),
                                yaml_serde::Value::Mapping(props),
                            ) = (ck, cv)
                                && ck_str == "properties"
                            {
                                return props.iter().find_map(|(pk, pv)| {
                                    if let (
                                        yaml_serde::Value::String(pk_str),
                                        yaml_serde::Value::String(pv_str),
                                    ) = (pk, pv)
                                        && pk_str == "name"
                                    {
                                        return Some(pv_str.as_str());
                                    }
                                    None
                                });
                            }
                            None
                        })
                    }
                    _ => None,
                };
                if let Some(name) = crs_name
                    && !matches!(
                        name,
                        "EPSG:4326" | "urn:ogc:def:crs:OGC:1.3:CRS84" | "CRS84"
                    )
                {
                    return Err(format!("Source input '{path}' declares unsupported CRS '{name}'; only WGS 84 (EPSG:4326) is accepted").into());
                }
            }
        }
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
        if let Some(scene) = &label.scene
            && !scene_ids.contains(scene.as_str())
        {
            return Err(
                format!("Label '{}' references unknown Scene '{scene}'", label.text).into(),
            );
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
        validate_spatial(
            &animation.translate,
            &animation.scale,
            sequence.duration_seconds,
            "animation",
        )?;
    }
    if let Some(transition) = &sequence.map_transition {
        validate_spatial(
            &transition.translate,
            &transition.scale,
            sequence.duration_seconds,
            "map transition",
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

fn validate_spatial(
    translate: &TransformKeyframes,
    scale: &[NumberKeyframe],
    duration_seconds: f64,
    context: &str,
) -> RenderResult<()> {
    validate_keyframes(
        &translate.x,
        duration_seconds,
        &format!("{context} translate.x"),
    )?;
    validate_keyframes(
        &translate.y,
        duration_seconds,
        &format!("{context} translate.y"),
    )?;
    validate_keyframes(scale, duration_seconds, &format!("{context} scale"))?;
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
    svg.push_str("</g>");
    for frame_label in &scene.labels {
        let label = frame_label.label;
        let opacity_attr = if frame_label.opacity < 1.0 {
            format!(" opacity=\"{:.3}\"", frame_label.opacity)
        } else {
            String::new()
        };
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#252525" font-family="sans-serif" font-size="16" font-weight="700"{}>{}</text>"##, label.x, label.y, opacity_attr, escape(&label.text)));
    }
    svg.push_str("</svg>");
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
