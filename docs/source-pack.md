# Source Pack format

A Source Pack is the reproducible input for one Scene. Every rendered layer and label names at least one Source Reference.

```
source-pack/
  scene.yaml
  sources.yaml
  data/          # local geographic inputs, such as GeoJSON
  materials/     # cached AI-generated non-factual textures
```

## `scene.yaml`

```yaml
title: Bangkok's Flat Delta
canvas:
  width: 3840
  height: 2160
background: "#edf0e7"
layers:
  - id: terrain
    kind: polygon
    fill: "#cbbf92"
    points: [[0, 2160], [0, 1000], [3840, 1200], [3840, 2160]]
    sources: [basin]
  - id: river
    kind: line
    stroke: "#668c94"
    stroke_width: 40
    points: [[0, 1300], [1600, 1180], [3840, 1400]]
    sources: [basin]
labels:
  - text: BANGKOK
    x: 2200
    y: 1200
    sources: [basin]
```

Supported layer kinds are `polygon` and `line`. Omit `canvas` to render at the default 3840×2160. The Scene source retains named layers; phase one exports one flattened PNG.

## `sources.yaml`

```yaml
sources:
  - id: basin
    title: Chao Phraya basin reference
    url: https://example.org/source
    license: ODbL-1.0
    retrieved: 2026-09-16
```

Each reference needs an ID, title, URL, license, and retrieval date. Use commercially suitable material and keep required attribution with the Source Pack.
