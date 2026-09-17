# Bangkok Scene Sequence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (\`- [ ]\`) syntax for tracking.

**Goal:** Render a deterministic, source-backed, three-stage Bangkok animation as numbered 4K/30 fps PNG frames.

**Architecture:** Keep static rendering intact and add a strict \`sequence.yaml\` model beside it. Reuse SVG composition and resvg per frame, applying only interpolated layer and global map transforms. Validate the entire pack before creating output, then write frames and a YAML handoff manifest.

**Tech Stack:** Rust 2024, \`serde\`, \`yaml_serde\`, \`resvg\`, integration tests using \`png\`.

**Spec:** \`docs/superpowers/specs/2026-09-17-bangkok-scene-sequence-design.md\`

## Global Constraints

- Preserve \`world-in-layers render <source-pack> <output.png>\` behavior and tests.
- Accept GeoJSON only in this milestone; document Shapefile/KML/KMZ GDAL import as deferred.
- Require strict YAML, known source references, title/URL/license/retrieval metadata, and explicit \`assume_crs: EPSG:…\` when CRS is absent.
- Produce 4K, 30 fps, 12-second output for the Bangkok pack; use lower dimensions, frame rates, and duration in automated tests.
- Do not add video encoding, an editor, a general camera system, scripting, physics, or new dependencies.

---

### Task 1: Define and validate strict sequence packs

**Files:**

- Modify: \`src/lib.rs:8-103\`
- Modify: \`tests/render_source_pack.rs\`

**Interfaces:**

- Produces: \`pub fn validate_sequence_pack(pack: impl AsRef<Path>) -> RenderResult<SequencePack>\`.
- Produces: strict \`SequencePack\`, \`SequenceScene\`, \`LayerAnimation\`, \`NumberKeyframe\`, \`TransformKeyframes\`, and \`MapTransition\`.
- Consumes: existing \`Layer\`, \`Label\`, \`Sources\`, and \`validate_references\`.

- [ ] **Step 1: Write the failing tests**

~~~rust
#[test]
fn rejects_unknown_sequence_fields_before_output_exists() {
    let pack = temporary_sequence_pack("unknown: value\n");
    let error = validate_sequence_pack(&pack).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn rejects_unknown_crs_without_an_explicit_assumption() {
    let pack = temporary_sequence_pack("assume_crs: invalid\n");
    let error = validate_sequence_pack(&pack).unwrap_err();
    assert!(error.to_string().contains("EPSG:"));
}
~~~

- [ ] **Step 2: Run RED**

Run: \`cargo test --test render_source_pack rejects_unknown_sequence_fields_before_output_exists rejects_unknown_crs_without_an_explicit_assumption\`

Expected: FAIL because \`validate_sequence_pack\` does not exist.

- [ ] **Step 3: Add the minimum strict model and validation**

~~~rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SequencePack {
    title: String,
    canvas: Canvas,
    bounds: [f64; 4],
    background: String,
    fps: u32,
    duration_seconds: f64,
    layers: Vec<Layer>,
    labels: Vec<Label>,
    scenes: Vec<SequenceScene>,
    #[serde(default)]
    animations: Vec<LayerAnimation>,
    map_transition: Option<MapTransition>,
}

pub fn validate_sequence_pack(pack: impl AsRef<Path>) -> RenderResult<SequencePack> {
    let pack = pack.as_ref();
    let sequence = yaml_serde::from_str(&fs::read_to_string(pack.join("sequence.yaml"))?)?;
    let sources = yaml_serde::from_str(&fs::read_to_string(pack.join("sources.yaml"))?)?;
    validate_sequence(&sequence, &sources)?;
    Ok(sequence)
}
~~~

Add \`#[serde(deny_unknown_fields)]\` to every static and sequence YAML input type. Validate positive fps/duration, unique and contiguous Scene ranges from zero to duration, unique layer IDs, monotonic keyframes in range, known source IDs, non-empty source metadata, and optional \`assume_crs\` values beginning \`EPSG:\`.

- [ ] **Step 4: Run GREEN and regression tests**

Run: \`cargo test --test render_source_pack\`

Expected: PASS.

- [ ] **Step 5: Commit**

~~~powershell
git add -- src/lib.rs tests/render_source_pack.rs
git commit -m "feat: validate strict sequence packs"
~~~

### Task 2: Apply constrained keyframe and map transforms

**Files:**

- Modify: \`src/lib.rs:104-267\`
- Modify: \`tests/render_source_pack.rs\`

**Interfaces:**

- Produces: \`fn frame_scene(sequence: &SequencePack, time: f64) -> FrameScene\`.
- Produces: \`fn interpolate(keyframes: &[NumberKeyframe], time: f64, default: f64) -> f64\`.
- Consumes: validated \`SequencePack\`; SVG composition receives \`FrameScene\`.

- [ ] **Step 1: Write the failing tests**

~~~rust
#[test]
fn interpolates_layer_opacity_at_the_frame_time() {
    assert_eq!(interpolate(&[keyframe(0.0, 0.0), keyframe(2.0, 1.0)], 1.0, 1.0), 0.5);
}

#[test]
fn sequence_frames_change_when_a_layer_fades_in() {
    let pack = temporary_animated_pack();
    assert_ne!(render_sequence_frame(&pack, 0).unwrap(), render_sequence_frame(&pack, 1).unwrap());
}
~~~

- [ ] **Step 2: Run RED**

Run: \`cargo test --test render_source_pack interpolates_layer_opacity_at_the_frame_time sequence_frames_change_when_a_layer_fades_in\`

Expected: FAIL because interpolation and sequence frame rendering do not exist.

- [ ] **Step 3: Implement the minimum frame projection**

~~~rust
fn interpolate(keyframes: &[NumberKeyframe], time: f64, default: f64) -> f64 {
    let Some(first) = keyframes.first() else { return default; };
    if time <= first.at { return first.value; }
    for pair in keyframes.windows(2) {
        if time <= pair[1].at {
            let fraction = (time - pair[0].at) / (pair[1].at - pair[0].at);
            return pair[0].value + (pair[1].value - pair[0].value) * fraction;
        }
    }
    keyframes.last().unwrap().value
}
~~~

Project each Layer with SVG \`opacity\`, \`translate(x y)\`, and \`scale(s)\`. Apply the optional global Map Transition as one outer SVG \`<g>\` around every Layer and Label. Static composition uses identity transforms.

- [ ] **Step 4: Run GREEN and add map-transition coverage**

~~~rust
#[test]
fn renders_a_valid_frame_during_a_map_transition() {
    assert!(render_sequence_frame(&temporary_transition_pack(), 1)
        .unwrap()
        .starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
}
~~~

Run: \`cargo test\`

Expected: PASS.

- [ ] **Step 5: Commit**

~~~powershell
git add -- src/lib.rs tests/render_source_pack.rs
git commit -m "feat: animate source pack layers"
~~~

### Task 3: Render frame sequences and manifests atomically

**Files:**

- Modify: \`src/lib.rs\`
- Modify: \`tests/render_source_pack.rs\`

**Interfaces:**

- Produces: \`pub fn render_sequence_pack(pack: impl AsRef<Path>, output: impl AsRef<Path>) -> RenderResult<()>\`.
- Produces: zero-padded PNG frames and \`render-manifest.yaml\`.
- Consumes: \`validate_sequence_pack\` and \`frame_scene\`.

- [ ] **Step 1: Write the failing output tests**

~~~rust
#[test]
fn renders_all_sequence_frames_and_a_manifest() {
    let pack = temporary_animated_pack();
    let output = pack.join("frames");
    render_sequence_pack(&pack, &output).unwrap();
    assert!(output.join("frame-000000.png").is_file());
    assert!(output.join("frame-000001.png").is_file());
    assert!(output.join("render-manifest.yaml").is_file());
}

#[test]
fn invalid_sequence_creates_no_output_directory() {
    let pack = temporary_sequence_pack("fps: 0\n");
    let output = pack.join("frames");
    assert!(render_sequence_pack(&pack, &output).is_err());
    assert!(!output.exists());
}
~~~

- [ ] **Step 2: Run RED**

Run: \`cargo test --test render_source_pack renders_all_sequence_frames_and_a_manifest invalid_sequence_creates_no_output_directory\`

Expected: FAIL because \`render_sequence_pack\` does not exist.

- [ ] **Step 3: Implement output only after validation**

~~~rust
pub fn render_sequence_pack(pack: impl AsRef<Path>, output: impl AsRef<Path>) -> RenderResult<()> {
    let sequence = validate_sequence_pack(pack.as_ref())?;
    fs::create_dir(output.as_ref())?;
    for frame in 0..(sequence.fps as usize * sequence.duration_seconds as usize) {
        fs::write(output.as_ref().join(format!("frame-{frame:06}.png")),
            render_sequence_frame_from_sequence(&sequence, frame)?)?;
    }
    fs::write(output.as_ref().join("render-manifest.yaml"), render_manifest(&sequence))?;
    Ok(())
}
~~~

Reject a pre-existing output directory. Render at \`frame / fps\` seconds. Use \`yaml_serde::to_string\` for a manifest containing title, fps, duration, frame count, frame pattern, ordered Scene ranges, and exact Attribution Card text.

- [ ] **Step 4: Run GREEN and full tests**

Run: \`cargo test\`

Expected: PASS.

- [ ] **Step 5: Commit**

~~~powershell
git add -- src/lib.rs tests/render_source_pack.rs
git commit -m "feat: render animated frame sequences"
~~~

### Task 4: Expose the command and document the contract

**Files:**

- Modify: \`src/main.rs\`
- Modify: \`tests/render_source_pack.rs\`
- Modify: \`docs/source-pack.md\`

**Interfaces:**

- Produces: \`world-in-layers render-sequence <source-pack> <output-dir>\`.
- Consumes: \`world_in_layers::render_sequence_pack\`.

- [ ] **Step 1: Write the failing CLI test**

~~~rust
#[test]
fn cli_renders_a_sequence_to_the_requested_directory() {
    let pack = temporary_animated_pack();
    let output = pack.join("frames");
    let status = Command::new(env!("CARGO_BIN_EXE_world-in-layers"))
        .args(["render-sequence", pack.to_str().unwrap(), output.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(output.join("render-manifest.yaml").is_file());
}
~~~

- [ ] **Step 2: Run RED**

Run: \`cargo test --test render_source_pack cli_renders_a_sequence_to_the_requested_directory\`

Expected: FAIL because the CLI rejects \`render-sequence\`.

- [ ] **Step 3: Add the CLI match arm and docs**

~~~rust
match command.to_string_lossy().as_ref() {
    "render" => world_in_layers::render_source_pack(pack, output),
    "render-sequence" => world_in_layers::render_sequence_pack(pack, output),
    _ => return usage(),
}
~~~

Document \`sequence.yaml\`, keyframes, output naming, the \`assume_crs: EPSG:4326\` option, strict validation, raw/normalized data directories, GeoJSON, deferred GDAL import, Attribution Card, and optional FFmpeg handoff.

- [ ] **Step 4: Run GREEN and formatting**

Run: \`cargo fmt --check; cargo test\`

Expected: both commands succeed.

- [ ] **Step 5: Commit**

~~~powershell
git add -- src/main.rs tests/render_source_pack.rs docs/source-pack.md
git commit -m "docs: describe sequence source packs"
~~~

### Task 5: Build the sourced Bangkok sequence pack

**Files:**

- Create: \`packs/bangkok-flat-delta/sequence.yaml\`
- Create: \`packs/bangkok-flat-delta/data/raw/README.md\`
- Create: \`packs/bangkok-flat-delta/data/geometry/chao-phraya.geojson\`
- Create: \`packs/bangkok-flat-delta/data/geometry/canals.geojson\`
- Create: \`packs/bangkok-flat-delta/data/geometry/coastline.geojson\`
- Create: \`packs/bangkok-flat-delta/data/geometry/flood-area.geojson\`
- Modify: \`packs/bangkok-flat-delta/sources.yaml\`
- Modify: \`packs/bangkok-flat-delta/README.md\`
- Test: \`tests/render_source_pack.rs\`

**Interfaces:**

- Consumes: strict sequence pack schema and \`render-sequence\`.
- Produces: 360 frames with the three named causal stages.

- [ ] **Step 1: Write the failing real-pack smoke test**

~~~rust
#[test]
fn bangkok_sequence_pack_validates() {
    validate_sequence_pack("packs/bangkok-flat-delta").unwrap();
}
~~~

- [ ] **Step 2: Run RED**

Run: \`cargo test --test render_source_pack bangkok_sequence_pack_validates\`

Expected: FAIL because the sequence pack does not exist.

- [ ] **Step 3: Add auditable local source geometry**

Keep original downloaded inputs in \`data/raw/\`; the README lists original filename, generated GeoJSON filename, Source Reference ID, and any \`assume_crs\`. Store WGS 84 GeoJSON in \`data/geometry/\`. Add licensed/retrieved Source References for river/canals, coastline/tidal boundary, elevation context, and flood/drainage evidence.

Create a 3840×2160, 30 fps, 12-second \`sequence.yaml\` with the shared base map and named Scenes \`rainfall-overload\` \`[0, 4)\`, \`river-back-pressure\` \`[4, 8)\`, and \`tide-blocked-outflow\` \`[8, 12)\`. Keyframe only the non-factual rain treatment, factual constraint/flood overlays, the constrained Map Transition, and the final Attribution Card. Use exactly: \`Geographic features are source-backed; timing is illustrative.\`

- [ ] **Step 4: Run GREEN, render, and verify**

Run: \`cargo test --test render_source_pack bangkok_sequence_pack_validates; cargo run -- render-sequence packs/bangkok-flat-delta .\\target\\bangkok-frames; cargo fmt --check; cargo test; git diff --check\`

Expected: the smoke test and checks pass; \`target/bangkok-frames\` contains 360 PNGs and \`render-manifest.yaml\`.

- [ ] **Step 5: Commit**

~~~powershell
git add -- packs/bangkok-flat-delta tests/render_source_pack.rs
git commit -m "feat: add Bangkok flooding scene sequence"
~~~
