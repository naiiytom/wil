use std::{
    fs,
    io::BufReader,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use world_in_layers::{
    NumberKeyframe, interpolate, render_sequence_frame, render_sequence_pack, render_source_pack,
    validate_sequence_pack,
};

fn temporary_pack() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "world-in-layers-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_source_pack(path: &PathBuf, layer_sources: &str) {
    fs::write(
        path.join("scene.yaml"),
        format!(
            r##"
title: Bangkok's Flat Delta
canvas:
  width: 320
  height: 180
background: "#edf0e7"
layers:
  - id: terrain
    kind: polygon
    fill: "#cbbf92"
    points: [[0, 180], [0, 90], [120, 55], [320, 95], [320, 180]]
    sources: {layer_sources}
  - id: river
    kind: line
    stroke: "#668c94"
    stroke_width: 16
    points: [[0, 110], [110, 100], [210, 120], [320, 105]]
    sources: {layer_sources}
labels:
  - text: "BANGKOK"
    x: 190
    y: 95
    sources: {layer_sources}
"##
        ),
    )
    .unwrap();
    fs::write(
        path.join("sources.yaml"),
        r#"
sources:
  - id: basin
    title: Chao Phraya basin reference
    url: https://example.com/basin
    license: ODbL-1.0
    retrieved: 2026-09-16
"#,
    )
    .unwrap();
}

fn temporary_sequence_pack(extra: &str) -> PathBuf {
    let pack = temporary_pack();
    fs::write(
        pack.join("sequence.yaml"),
        format!(
            r##"
title: Animated Bangkok
canvas:
  width: 320
  height: 180
bounds: [100.0, 13.0, 101.0, 14.0]
background: "#edf0e7"
fps: 2
duration_seconds: 2.0
layers:
  - id: terrain
    kind: polygon
    fill: "#cbbf92"
    points: [[0, 180], [0, 0], [320, 0], [320, 180]]
    sources: [basin]
labels: []
scenes:
  - id: rainfall
    start: 0.0
    end: 2.0
{extra}"##
        ),
    )
    .unwrap();
    fs::write(
        pack.join("sources.yaml"),
        r#"
sources:
  - id: basin
    title: Chao Phraya basin reference
    url: https://example.com/basin
    license: ODbL-1.0
    retrieved: 2026-09-16
"#,
    )
    .unwrap();
    pack
}

fn keyframe(at: f64, value: f64) -> NumberKeyframe {
    NumberKeyframe { at, value }
}

fn temporary_animated_pack() -> PathBuf {
    temporary_sequence_pack(
        r#"animations:
  - layer: terrain
    opacity: [{at: 0.0, value: 0.0}, {at: 2.0, value: 1.0}]
"#,
    )
}

fn temporary_transition_pack() -> PathBuf {
    temporary_sequence_pack(
        r#"map_transition:
  translate:
    x: [{at: 0.0, value: 0.0}, {at: 2.0, value: 8.0}]
    y: [{at: 0.0, value: 0.0}, {at: 2.0, value: 4.0}]
  scale: [{at: 0.0, value: 1.0}, {at: 2.0, value: 1.1}]
"#,
    )
}

#[test]
fn renders_all_sequence_frames_and_a_manifest() {
    let pack = temporary_animated_pack();
    let output = pack.join("frames");

    render_sequence_pack(&pack, &output).unwrap();

    assert!(output.join("frame-000000.png").is_file());
    assert!(output.join("frame-000001.png").is_file());
    assert!(output.join("render-manifest.yaml").is_file());
    let manifest = fs::read_to_string(output.join("render-manifest.yaml")).unwrap();
    assert!(manifest.contains("Animated Bangkok"));
    assert!(manifest.contains("frame_count: 4"));
    assert!(manifest.contains("frame_pattern: frame-%06d.png"));
    assert!(manifest.contains("Geographic features are source-backed; timing is illustrative."));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn invalid_sequence_creates_no_output_directory() {
    let pack = temporary_sequence_pack("fps: 0\n");
    let output = pack.join("frames");

    assert!(render_sequence_pack(&pack, &output).is_err());
    assert!(!output.exists());
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn interpolates_layer_opacity_at_the_frame_time() {
    assert_eq!(
        interpolate(&[keyframe(0.0, 0.0), keyframe(2.0, 1.0)], 1.0, 1.0),
        0.5
    );
}

#[test]
fn sequence_frames_change_when_a_layer_fades_in() {
    let pack = temporary_animated_pack();
    assert_ne!(
        render_sequence_frame(&pack, 0).unwrap(),
        render_sequence_frame(&pack, 1).unwrap()
    );
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn renders_a_valid_frame_during_a_map_transition() {
    let pack = temporary_transition_pack();
    assert!(
        render_sequence_frame(&pack, 1)
            .unwrap()
            .starts_with(&[137, 80, 78, 71, 13, 10, 26, 10])
    );
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rejects_duplicate_animation_target_layers() {
    let pack = temporary_sequence_pack(
        r#"animations:
  - layer: terrain
    opacity: [{at: 0.0, value: 0.0}]
  - layer: terrain
    scale: [{at: 0.0, value: 1.0}]
"#,
    );
    let error = validate_sequence_pack(&pack).unwrap_err();
    assert!(error.to_string().contains("animation target layers"));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rejects_unknown_sequence_fields_before_output_exists() {
    let pack = temporary_sequence_pack("unknown: value\n");
    let error = validate_sequence_pack(&pack).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rejects_unknown_crs_without_an_explicit_assumption() {
    let pack = temporary_sequence_pack("assume_crs: invalid\n");
    let error = validate_sequence_pack(&pack).unwrap_err();
    assert!(error.to_string().contains("EPSG:"));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn writes_a_png_for_a_source_backed_scene() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[basin]");
    let output = pack.join("render.png");

    render_source_pack(&pack, &output).unwrap();

    let png = fs::read(&output).unwrap();
    assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn defaults_a_scene_render_to_4k() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[basin]");
    let scene = fs::read_to_string(pack.join("scene.yaml"))
        .unwrap()
        .replace("canvas:\n  width: 320\n  height: 180\n", "");
    fs::write(pack.join("scene.yaml"), scene).unwrap();
    let output = pack.join("render.png");

    render_source_pack(&pack, &output).unwrap();

    let png = fs::read(&output).unwrap();
    assert_eq!(&png[16..24], &[0, 0, 15, 0, 0, 0, 8, 112]);
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rasterizes_scene_labels() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[basin]");
    let output = pack.join("render.png");

    render_source_pack(&pack, &output).unwrap();

    let decoder = png::Decoder::new(BufReader::new(fs::File::open(&output).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert!(
        pixels[..info.buffer_size()]
            .chunks_exact(4)
            .any(|pixel| pixel[0] < 50 && pixel[1] < 50 && pixel[2] < 50)
    );
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn casts_a_shadow_for_a_lifted_paper_layer() {
    let pack = temporary_pack();
    fs::write(
        pack.join("scene.yaml"),
        r##"
canvas:
  width: 120
  height: 80
background: "#ffffff"
layers:
  - id: paper-layer
    kind: polygon
    fill: "#d3c797"
    lift: 12
    points: [[20, 20], [100, 20], [100, 50], [20, 50]]
    sources: [basin]
"##,
    )
    .unwrap();
    fs::write(
        pack.join("sources.yaml"),
        r#"
sources:
  - id: basin
    title: Test source
    url: https://example.com/source
    license: CC0-1.0
    retrieved: 2026-09-16
"#,
    )
    .unwrap();
    let output = pack.join("render.png");

    render_source_pack(&pack, &output).unwrap();

    let decoder = png::Decoder::new(BufReader::new(fs::File::open(&output).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut pixels).unwrap();
    let shadow = &pixels[(56 * 120 + 60) * 4..(56 * 120 + 61) * 4];
    assert!(shadow[0] < 230 && shadow[1] < 230 && shadow[2] < 230);
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn renders_a_local_geojson_line() {
    let pack = temporary_pack();
    fs::create_dir_all(pack.join("data")).unwrap();
    fs::write(
        pack.join("scene.yaml"),
        r##"
canvas:
  width: 120
  height: 80
bounds: [100.0, 13.0, 101.0, 14.0]
background: "#ffffff"
layers:
  - id: river
    kind: line
    stroke: "#334455"
    stroke_width: 10
    points: []
    geojson: data/river.geojson
    sources: [basin]
"##,
    )
    .unwrap();
    fs::write(
        pack.join("sources.yaml"),
        r#"
sources:
  - id: basin
    title: Test source
    url: https://example.com/source
    license: CC0-1.0
    retrieved: 2026-09-16
"#,
    )
    .unwrap();
    fs::write(
        pack.join("data/river.geojson"),
        r#"{"type":"Feature","geometry":{"type":"LineString","coordinates":[[100.1,13.9],[100.9,13.1]]},"properties":{}}"#,
    )
    .unwrap();
    let output = pack.join("render.png");

    render_source_pack(&pack, &output).unwrap();

    let decoder = png::Decoder::new(BufReader::new(fs::File::open(&output).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut pixels).unwrap();
    let river = &pixels[(40 * 120 + 60) * 4..(40 * 120 + 61) * 4];
    assert!(river[0] < 100 && river[1] < 100 && river[2] < 100);
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rejects_layers_without_a_source_reference() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[]");

    let error = render_source_pack(&pack, pack.join("render.png")).unwrap_err();

    assert!(error.to_string().contains("source reference"));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn rejects_an_unknown_layer_kind() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[basin]");
    let scene = fs::read_to_string(pack.join("scene.yaml"))
        .unwrap()
        .replace("kind: polygon", "kind: unknown");
    fs::write(pack.join("scene.yaml"), scene).unwrap();

    let error = render_source_pack(&pack, pack.join("render.png")).unwrap_err();

    assert!(error.to_string().contains("unknown layer kind"));
    fs::remove_dir_all(pack).unwrap();
}

#[test]
fn cli_renders_to_the_requested_path() {
    let pack = temporary_pack();
    write_source_pack(&pack, "[basin]");
    let output = pack.join("scene.png");

    let status = Command::new(env!("CARGO_BIN_EXE_world-in-layers"))
        .args(["render", pack.to_str().unwrap(), output.to_str().unwrap()])
        .status()
        .unwrap();

    assert!(status.success());
    assert!(
        fs::read(&output)
            .unwrap()
            .starts_with(&[137, 80, 78, 71, 13, 10, 26, 10])
    );
    fs::remove_dir_all(pack).unwrap();
}
