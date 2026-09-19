# Orchestrate Full Episodes via Declarative Manifests and render-episode

World in Layers will coordinate multi-sequence episodes via an `episode.yaml` manifest and a dedicated `render-episode <pack> <output> [--jobs N]` CLI command. Rather than monolithic project files or external scripts, `episode.yaml` declaratively orders independent sequence packs, validates all child packs and shared geometries atomically upfront, parallelizes frame rendering into structured subdirectories, and outputs an `episode-manifest.yaml` mapping rendered frame ranges to script narrative sections.
