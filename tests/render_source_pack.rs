use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use world_in_layers::render_source_pack;

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
