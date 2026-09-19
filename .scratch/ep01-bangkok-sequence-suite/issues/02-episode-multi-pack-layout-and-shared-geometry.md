# Episode Multi-Pack Layout and Shared Geometry

Type: grilling
Status: resolved
Blocked by: none

## Question

How should `packs/ep01-bangkok/` be structured to support multiple independent Scene Sequences while cleanly sharing common geographic datasets (Chao Phraya river, coastline, elevation contours) without compromising offline reproducibility or strict pack validation?

## Answer

`packs/ep01-bangkok/` will be structured as an episode workspace containing shared assets and five independent sequence packs:

1. **Directory Tree**:
   ```text
   packs/ep01-bangkok/
   ├── _shared/
   │   ├── data/geometry/      # chao-phraya, coastline, low-elevation contours
   │   └── sources.yaml        # master episode source reference registry
   ├── 01-delta-exposure/
   ├── 02-canal-city/
   ├── 03-subsidence-cutaway/
   ├── 04-three-pressures/
   └── 05-drainage-capacity/
   ```
2. **Relative Geometry Path Resolution**: Sequences reference shared geometry files using standard relative paths (e.g. `"../_shared/data/geometry/chao-phraya.geojson"`). Because `resolve_geojson_layers` joins paths directly to the pack path via `pack.join(path)`, standard relative paths function out of the box with zero engine modifications or schema bloat.
3. **Self-Contained Local `sources.yaml`**: Each sequence pack contains its own `sources.yaml` that extracts only the citations relevant to its layers and labels from `_shared/sources.yaml`. This keeps each sequence pack 100% validatable and renderable in isolation and guarantees that generated `render-manifest.yaml` Attribution Cards cite only the datasets used in that scene sequence.
4. **Baseline Preservation**: `packs/bangkok-flat-delta` is retained untouched as an immutable regression baseline for automated test suites.
