use std::{collections::HashSet, error::Error, fs, path::Path};

use serde::Deserialize;

pub type RenderResult<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Scene {
    #[serde(default)]
    canvas: Canvas,
    background: String,
    layers: Vec<Layer>,
    #[serde(default)]
    labels: Vec<Label>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct Layer {
    id: String,
    kind: String,
    #[serde(default)]
    fill: String,
    #[serde(default)]
    stroke: String,
    #[serde(default)]
    stroke_width: f32,
    points: Vec<[f32; 2]>,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Deserialize)]
struct Label {
    text: String,
    x: f32,
    y: f32,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Deserialize)]
struct Sources {
    sources: Vec<SourceReference>,
}

#[derive(Deserialize)]
struct SourceReference {
    id: String,
    title: String,
    url: String,
    license: String,
    retrieved: String,
}

pub fn render_source_pack(pack: impl AsRef<Path>, output: impl AsRef<Path>) -> RenderResult<()> {
    let pack = pack.as_ref();
    let scene: Scene = yaml_serde::from_str(&fs::read_to_string(pack.join("scene.yaml"))?)?;
    let sources: Sources = yaml_serde::from_str(&fs::read_to_string(pack.join("sources.yaml"))?)?;
    validate(&scene, &sources)?;

    let svg = compose_svg(&scene);
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())?;
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

fn validate(scene: &Scene, sources: &Sources) -> RenderResult<()> {
    let source_ids: HashSet<&str> = sources
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
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
    }
    for layer in &scene.layers {
        if !matches!(layer.kind.as_str(), "polygon" | "line") {
            return Err(format!(
                "Layer '{}' has unknown layer kind '{}'",
                layer.id, layer.kind
            )
            .into());
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
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><rect width="100%" height="100%" fill="{}"/>"#,
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
            "none"
        } else {
            &layer.stroke
        };
        match layer.kind.as_str() {
            "polygon" => svg.push_str(&format!(r#"<polygon id="{}" points="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#, escape(&layer.id), points, fill, stroke, layer.stroke_width)),
            "line" => svg.push_str(&format!(r#"<polyline id="{}" points="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#, escape(&layer.id), points, stroke, layer.stroke_width)),
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
