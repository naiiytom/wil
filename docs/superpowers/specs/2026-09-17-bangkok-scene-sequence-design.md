# Bangkok Scene Sequence Design

## Goal

Render a 10–15 second, 4K, 30 fps source-backed Bangkok Scene Sequence as numbered PNG frames. Three named Scenes explain rainfall overload, river back-pressure, and tide-blocked discharge; geographic features are source-backed while timing is illustrative.

## Scope

The existing static `render <source-pack> <output.png>` command remains unchanged. A new `render-sequence <source-pack> <output-dir>` command validates a sequence before emitting frames and a render manifest. FFmpeg assembly, video encoding, a timeline editor, scripting, physics, and generalized camera animation are out of scope.

## Source Pack Contract

Each sequence pack contains `sequence.yaml`, `sources.yaml`, `data/raw/`, and `data/geometry/`. `sequence.yaml` contains the canvas, bounds, background, shared layers and labels, three ordered Scene ranges, layer keyframes, and at most one constrained global Map Transition. A Scene range is `[start, end)` in seconds; the first pack uses three four-second ranges at 30 fps.

`sequence.yaml` is strict: unknown fields fail deserialization. Layers use the existing polygon and line forms. A layer may have keyframes for `opacity`, `translate`, and `scale`; values are linearly interpolated between adjacent timestamps. The optional global Map Transition uses the same interpolation and transforms all layers together, preserving the same geographic features between Scenes. No arbitrary transform, script, physics, or free camera fields are accepted.

`sources.yaml` remains strict. Every source reference requires `id`, `title`, `url`, `license`, and `retrieved`; every layer and label references at least one known source. Local raw inputs and generated GeoJSON are retained in the pack. Imported GeoJSON is WGS 84 longitude/latitude. A source whose input lacks CRS metadata is rejected unless its manifest has an explicit `assume_crs: EPSG:…` value; the pack documentation lists supported values and their meaning. A boolean CRS assumption is not valid.

The first vertical slice accepts locally curated GeoJSON. A deferred GDAL-backed importer will convert Shapefile, KML, and KMZ to the canonical GeoJSON geometry form and will report a clear prerequisite error when GDAL is unavailable.

## Renderer Behavior

`render-sequence` loads and validates all sequence and source data before creating the output directory or writing a frame. It renders `fps × duration_seconds` PNGs named with zero-padded frame numbers and writes `render-manifest.yaml` containing title, fps, duration, frame count, frame naming pattern, Scene ranges, and the Attribution Card text.

The first Bangkok sequence has one continuous base map and three four-second Scenes:

1. `rainfall-overload` reveals non-factual rain treatment and a source-backed local drainage/flood overlay.
2. `river-back-pressure` retains the base map and adds a source-backed Chao Phraya constraint.
3. `tide-blocked-outflow` retains the map, adds a source-backed coastal/tidal constraint, reveals the combined flood area, and ends with the Attribution Card: “Geographic features are source-backed; timing is illustrative.”

The render output is deterministic for the same pack. Failure leaves no output directory or frame files.

## Pack Content

The Bangkok pack replaces hand-authored factual geometry with locally retained, curated GeoJSON for its river/canals, coastline/tidal boundary, elevation context, and flood/drainage overlays. Every geometry has a Source Reference with commercial-use license and retrieval date. Decorative rain treatment is a non-factual paper-diorama Layer with a factual source reference for its explanatory claim.

## CLI and Documentation

The CLI accepts only `render` and `render-sequence`; extra or missing arguments return usage failure. `docs/source-pack.md` documents static packs, sequence packs, GeoJSON as the current input, the deferred GDAL importer, CRS rules including the explicit `assume_crs: EPSG:…` option list, validation rules, output files, and FFmpeg as an optional external handoff.

## Tests

Integration tests create temporary packs and use public library APIs plus the compiled CLI. They prove sequence rendering writes the expected number of PNG frames and manifest; layer opacity changes pixels across time; global map transitions preserve output validity; malformed or unknown YAML fails; and missing source metadata, invalid CRS assumptions, missing references, and invalid keyframes fail before output is created. Existing static rendering tests remain unchanged.
