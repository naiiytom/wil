# Source Pack format

A Source Pack is reproducible input for a static Scene or a Scene sequence. Every rendered layer and label names at least one Source Reference. YAML is strict: unknown keys are rejected.

```
source-pack/
  scene.yaml                 # static pack
  sequence.yaml              # sequence pack (use one entry point per render)
  sources.yaml
  data/
    raw/                     # downloaded originals; retain provenance
    geometry/                # normalized WGS 84 GeoJSON used by the pack
  materials/                 # cached AI-generated non-factual textures
```

## Static packs

Run `world-in-layers render <source-pack> <output.png>`. A static pack has `scene.yaml` and `sources.yaml`:

```yaml
title: Bangkok's Flat Delta
canvas: {width: 3840, height: 2160} # optional; defaults to 3840x2160
bounds: [100.3278772, 13.2191019, 100.9386039, 13.9551693] # west, south, east, north
background: "#edf0e7"
layers:
  - id: terrain
    kind: polygon             # polygon or line
    fill: "#cbbf92"
    points: [[0, 2160], [0, 1000], [3840, 1200], [3840, 2160]]
    sources: [basin]
labels:
  - text: BANGKOK
    x: 2200
    y: 1200
    sources: [basin]
```

Layers support `id`, `kind`, `fill`, `stroke`, `stroke_width`, `lift`, `points`, optional `geojson`, and `sources`; labels support `text`, `x`, `y`, and `sources`.

GeoJSON is the current local geographic-input format. For a static layer, set `geojson: data/geometry/river.geojson` and provide `bounds`; `LineString` and `Polygon`, either directly or in one `Feature`, are projected from longitude/latitude into the canvas. Keep downloaded raw data in `data/raw/` and normalized, auditable WGS 84 GeoJSON in `data/geometry/`.

## Sequence packs

Run `world-in-layers render-sequence <source-pack> <output-dir>`. A sequence pack has `sequence.yaml` and `sources.yaml`. It validates all inputs before creating the output directory; output must not already exist.

```yaml
title: Animated Bangkok
canvas: {width: 3840, height: 2160}
bounds: [100.3278772, 13.2191019, 100.9386039, 13.9551693]
background: "#edf0e7"
fps: 30
duration_seconds: 12.0
assume_crs: EPSG:4326        # only when input CRS is absent
layers:
  - id: terrain
    kind: polygon
    fill: "#cbbf92"
    points: [[0, 2160], [0, 0], [3840, 0], [3840, 2160]]
    sources: [basin]
labels: []
scenes:
  - id: rainfall-overload
    start: 0.0
    end: 4.0
animations:
  - layer: terrain
    opacity: [{at: 0.0, value: 0.0}, {at: 4.0, value: 1.0}]
    translate:
      x: [{at: 0.0, value: 0.0}, {at: 4.0, value: 20.0}]
      y: []
    scale: [{at: 0.0, value: 1.0}, {at: 4.0, value: 1.05}]
map_transition:
  translate:
    x: []
    y: []
  scale: []
```

The full `sequence.yaml` key set is `title`, `canvas`, `bounds`, `background`, `fps`, `duration_seconds`, `layers`, `labels`, `scenes`, optional `animations`, optional `map_transition`, and optional `assume_crs`. Each keyframe is `{at: seconds, value: number}`. Keyframes are finite, strictly increasing, and within the duration. Scenes have unique IDs and contiguous `[start, end)` ranges spanning zero through `duration_seconds`; layer IDs and animation targets are unique. `fps` and duration are positive and their product is a whole frame count. Every layer and label has known source IDs.

`assume_crs: EPSG:4326` explicitly declares WGS 84 only when source data has no CRS metadata. Use the documented source CRS whenever it is available; `assume_crs` values must begin with `EPSG:`. The same optional key may be attached to a source entry in `sources.yaml`.

The render writes zero-padded PNG frames such as `frame-000000.png` and `render-manifest.yaml`. The manifest records title, fps, duration, frame count, `frame-%06d.png`, Scene ranges, and the Attribution Card: `Geographic features are source-backed; timing is illustrative.`

## Sources and deferred imports

```yaml
sources:
  - id: basin
    title: Chao Phraya basin reference
    url: https://example.org/source
    license: ODbL-1.0
    retrieved: 2026-09-16
    assume_crs: EPSG:4326     # optional, only if metadata is absent
```

Each Source Reference requires `id`, `title`, `url`, `license`, and `retrieved`. Preserve required attribution with the pack and the generated Attribution Card.

Shapefile, KML, and KMZ are deferred. Import them with GDAL into normalized WGS 84 GeoJSON before using them; GDAL is therefore a prerequisite for those formats, not a renderer dependency. After rendering, FFmpeg may optionally assemble the numbered PNGs into video; encoding is an external handoff and not part of this command.
