# Aggregated Attribution Card Format

Type: grilling
Status: resolved
Blocked by: 06

## Question

How should the engine aggregate and deduplicate `sources.yaml` citations across all five Scene Sequences to render a unified, compliant closing Attribution Card scene for Episode 1?

## Answer

The Episode 1 Attribution Card format balances visual screen elegance with exhaustive legal and editorial attribution:

1. **Rendering Surface**:
   The closing scene of Sequence 05 (`packs/ep01-bangkok/05-drainage-capacity/`) serves as the visual Attribution Card presentation for the episode. `render-episode` aggregates all source citations into the top-level `episode-manifest.yaml`.
2. **On-Screen Compact Visual Presentation**:
   - Canonical Thesis: *"Geographic features are source-backed; timing is illustrative."*
   - Institutional Credit Roll: Compactly acknowledges primary source providers (*Royal Thai Survey Department, JICA, Chulalongkorn University, Bangkok Metropolitan Administration (DDS), OpenStreetMap Contributors, European Commission JRC*).
3. **Structured Categorization in Manifest**:
   `episode-manifest.yaml` deduplicates citations by unique `id` and structures them into three editorial categories for video description and auditing:
   - *Geographic & Elevation Baselines* (Copernicus DEM, HydroSHEDS, OpenStreetMap)
   - *Historical & Morphological Surveys* (Royal Thai Survey Dept 1896, Bradley 1870, GHSL)
   - *Hydrological & Infrastructure Records* (BMA DDS, JICA, WMO)
   Each entry includes title, URL, license, and retrieval date.
4. **Strict Cross-Pack Consistency Validation**:
   During `render-episode` pre-validation, if a source `id` is declared across multiple sequence packs, its `title`, `url`, and `license` must match identically. Any discrepancies fail validation prior to rendering.
