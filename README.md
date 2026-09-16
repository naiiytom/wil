# World in Layers

A Windows-first Rust CLI for rendering source-backed Paper Diorama Scenes as 16:9 PNGs for the World in Layers YouTube channel.

## Use

```powershell
cargo test
cargo run -- render path\to\source-pack path\to\scene.png
```

The current compositor supports source-backed polygons, lines, and labels. It deliberately does not animate, fetch data, encode video, provide an editor, or call AI services.

## Source packs

Each Source Pack contains `scene.yaml` and `sources.yaml`. Keep local GeoJSON/SVG/PNG inputs and cached AI Materials beside them; rendering stays offline.

See [the Source Pack format](docs/source-pack.md) for the YAML contract.
