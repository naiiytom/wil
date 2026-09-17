# Retained input provenance

The renderer reads only the normalized WGS 84 GeoJSON in `../geometry`; no network request is made while rendering. Each normalized feature below was selected from the identified local source extract, retaining its source identifier and a WGS 84 (`EPSG:4326`) coordinate order of longitude, latitude.

| Original input filename | Normalized geometry | Source ID | CRS assumption |
| --- | --- | --- | --- |
| `osm-relation-227317-chao-phraya.geojson` | `../geometry/chao-phraya.geojson` | `osm-water` | None; OSM GeoJSON is WGS 84. |
| `osm-way-55931363-khlong-kudi-chin.geojson` | `../geometry/canals.geojson` | `osm-water` | None; OSM GeoJSON is WGS 84. |
| `osm-way-255391976-gulf-coastline.geojson` | `../geometry/coastline.geojson` | `osm-coastline` | None; OSM GeoJSON is WGS 84. |
| `bangkok-boundary.geojson` | _(retained for provenance; not rendered)_ | `osm-bangkok-boundary` | None; OSM GeoJSON is WGS 84. |
| `srtm-bangkok-sub5m-elevation.geojson` | `../geometry/low-elevation-zone.geojson` | `srtm-elevation` | None; curated from SRTM in WGS 84. |
| `jrc-gsw-bangkok-occurrence-gte50.geojson` | `../geometry/flood-extent.geojson` | `jrc-global-surface-water` | None; curated from JRC in WGS 84. |

The river, canal, coastline, and study-extent coordinates are selected from the cited OpenStreetMap inputs, rather than redrawn. The elevation context polygon was derived from the SRTM 1-arc-second DEM (pixels ≤ 5 m above EGM96 geoid). The flood extent polygon was derived from the JRC Global Surface Water v1.4 occurrence layer (occurrence ≥ 50%). Neither the elevation zone nor the flood extent represents a surveyed or event-specific flood boundary.
