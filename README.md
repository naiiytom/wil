# World in Layers

A Windows-first Rust CLI for rendering source-backed Paper Diorama Scenes as 16:9 PNGs for the World in Layers YouTube channel.

## Use

```powershell
cargo test
cargo run -- render path\to\source-pack path\to\scene.png
cargo run -- render-sequence path\to\sequence-pack path\to\output-dir
```

The compositor supports source-backed polygons, lines, and labels. It does not fetch data, encode video, provide an editor, or call AI services.

`render` writes a single PNG. `render-sequence` validates the full sequence pack, then writes numbered PNG frames and a `render-manifest.yaml`. FFmpeg assembly is out of scope.

## Source packs

Static packs contain `scene.yaml` and `sources.yaml`. Sequence packs additionally contain `sequence.yaml` with canvas, bounds, named Scene ranges, layer keyframes, and an optional global Map Transition. Keep local GeoJSON inputs and raw source files beside them; rendering stays offline.

See [the Source Pack format](docs/source-pack.md) for the YAML contract.
