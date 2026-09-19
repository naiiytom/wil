# Domain Docs

How engineering skills consume this repo's domain documentation.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root: authoritative project terminology and definitions.
- **`docs/adr/`**: architectural decision records for static rendering, keyframe sequences, and source packs.

## File structure

Single-context repo:

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/
│   │   ├── 0001-static-cli-mvp.md
│   │   ├── 0001-normalized-source-geometry.md
│   │   ├── 0002-keyframed-frame-sequences.md
│   │   └── 0003-source-pack-scene-sequences.md
│   └── agents/
└── src/
```

## Use the glossary's vocabulary

Always use terms defined in `CONTEXT.md`:
- Creator (not end user/customer)
- Episode (not project/montage)
- Paper Diorama (not visual component)
- Scene (not shot/animation)
- Scene Sequence (not sequence/montage)
- Map Transition
- Layer
- Source-backed
- Source Pack
- Source Geometry
- Source Reference
- Attribution Card
- Explanatory Timing
- AI Material
- Scene Render
- Label
