# Normalize Source Geometry to GeoJSON

Source Packs store GeoJSON as their rendering format. A later GDAL-backed importer will convert Shapefile, KML, and KMZ inputs into it; the first vertical slice accepts GeoJSON directly. Inputs with a declared CRS are normalized to WGS 84; an otherwise rejected input may use an explicit `assume_crs: EPSG:…` manifest value, so geographic assumptions remain auditable rather than implicit.
