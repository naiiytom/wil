# Map: Episode 1 Scene Sequence Suite (Why Bangkok Keeps Flooding)

## Destination

Deliver the complete specification, multi-pack layout, and deterministic 4K parallel rendering capability for all required Scene Sequences comprising the full ~8-minute Episode 1 (*Why Bangkok Keeps Flooding*).

## Notes

- **Domain**: Paper Diorama animation engine for the World in Layers channel. Always use canonical vocabulary from [`CONTEXT.md`](../../CONTEXT.md).
- **Consult Skills**: `grilling`, `domain-modeling`, `research`, `prototype`.
- **Standing Preferences**:
  - Offline reproducibility: all geometries and sources must be self-contained in packs.
  - Strict validation: `#[serde(deny_unknown_fields)]`, complete source citations, explicit CRS assumptions.
  - Parallel performance: parallelize 4K frame generation across CPU threads.
  - Scope discipline: render deterministic 4K PNG sequences and YAML manifests; external video encoding is out of scope.

## Decisions so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Parallel Frame Rendering Architecture](issues/01-parallel-frame-rendering.md): Use Rayon work-stealing with shared fontdb and optional --jobs flag for deterministic atomic 4K frame generation.
- [Episode Multi-Pack Layout and Shared Geometry](issues/02-episode-multi-pack-layout-and-shared-geometry.md): Organize episode under packs/ep01-bangkok/ with _shared/ geometry referenced via relative paths and local sources.yaml.
- [Historical Canal and Urban Paving Data Sources](issues/03-historical-canal-and-urban-paving-data-sources.md): Identified open historical canal and urban expansion GIS/map datasets with commercial-use paths.
- [Subsidence Cross-Section Cutaway Design](issues/04-subsidence-cross-section-cutaway-design.md): Composed subterranean cutaway via canvas-space polygon layers and keyframed vertical compaction.
- [Drainage System Capacity Indicators](issues/05-drainage-system-capacity-indicators.md): Model capacity saturation via canal color cross-fades, tidal collision lines, and floodgate closure barriers.
- [Master Episode Sequence Manifest](issues/06-master-episode-sequence-manifest.md): Declare episode.yaml conductor and render-episode CLI command emitting structured frames and episode-manifest.yaml.
- [Aggregated Attribution Card Format](issues/07-aggregated-attribution-card.md): Render closing attribution card in Sequence 05 and compile categorized deduplicated bibliography into episode-manifest.yaml.


## Not yet specified

<!-- see "Fog of war": in-scope fog you can't ticket yet; graduates as the frontier advances -->

- **Automated GDAL batch ingestion**: CLI importer converting raw Shapefile/KML/raster datasets to canonical WGS 84 GeoJSON once datasets are curated.
- **GeoJSON parse caching**: Shared in-memory or binary cache to avoid reparsing large delta GeoJSON files across concurrent sequence renders.

## Out of scope

<!-- see "Out of scope": work ruled beyond the destination; closed, never graduates -->

- **In-engine video encoding**: Direct FFmpeg invocation or MP4/MOV output from the Rust CLI.
- **Interactive timeline editor**: Web or desktop graphical user interfaces for scene choreography.
- **Other episode topics**: Topics from the topic bank outside Episode 1 (e.g. Strait of Malacca, Singapore food logistics).
- **Audio and narration production**: Voiceover recording, subtitle generation, or audio track synchronization.
