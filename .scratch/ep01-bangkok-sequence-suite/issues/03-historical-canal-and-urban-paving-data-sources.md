# Historical Canal and Urban Paving Data Sources

Type: research
Status: resolved
Blocked by: none

## Question

What open, commercially usable historical map archives and GIS datasets exist for Bangkok's 19th-century canal (khlong) network and 20th-century urban expansion footprint that can be digitized into WGS 84 GeoJSON with compliant source references?

## Answer

Comprehensive research has been completed and documented in [`docs/research/bangkok-historical-canals-and-urban-paving.md`](../../../docs/research/bangkok-historical-canals-and-urban-paving.md).

### Summary of Identified Viable Datasets & Paths:

1. **19th-Century Canal Network (~1850–1900)**:
   - **Primary Benchmark**: **Royal Thai Survey Department 1896 Map of Bangkok (*Phaenthi Krung Thep*)** (`rsd-bangkok-1896`). An ultra-high-resolution 10,787 × 15,549 px scan (32.7 MB) curated by NLA Trove and Wikimedia Commons. Licensed under **Public Domain** (`PD-old-70` / `Public Domain Mark 1.0`). Digitized via QGIS Thin Plate Spline georeferencing to WGS 84 (`EPSG:4326`).
   - **Secondary Moat Baseline**: **Dr. Dan Beach Bradley 1870 City Map of Bangkok** (`bradley-bangkok-1870`), Singapore Mission Press. High-res scan (2,902 × 3,664 px). **Public Domain** (`PD-old-100`).
   - **Modern Active Baseline & Filled Canal Traces**: **OpenStreetMap** (`osm-waterways-bangkok`, **ODbL-1.0**) via Overpass QL filtering `waterway=canal`, `historic=canal`, `abandoned:waterway=canal`, and `tunnel=culvert` directly to WGS 84 GeoJSON; alongside the **Bangkok Metropolitan Administration (BMA) Canal Network GIS** (`bma-canal-gis`, **Open Government Data License of Thailand** via `data.go.th`).

2. **20th-Century Urban Expansion & Paving Footprint (Impervious Surfaces)**:
   - **Year-by-Year Growth (1985–2015)**: **DLR World Settlement Footprint (WSF) Evolution** (`dlr-wsf-evolution`). 30-meter resolution annual settlement arrival raster. Licensed under **CC BY 4.0** (commercial use permitted). Vectorized via `gdal_calc.py` epoch masks and `gdal_polygonize.py` directly into WGS 84 GeoJSON.
   - **Long-Term Multi-Temporal Anchor (1975–2025)**: **EC JRC Global Human Settlement Layer Built-Up Surface (GHS-BUILT-S R2023A)** (`jrc-ghsl-builts`). 100m/10m multi-temporal raster under **CC BY 4.0** (directly matching the provenance of JRC Global Surface Water already in `packs/bangkok-flat-delta`).
   - **Mid-Century Baseline (1954–1962)**: **U.S. Army Map Service (AMS) Series L708 Sheet 5152-III & Series L9013/L9013S Town Plans** (`ams-bangkok-1959`), archived at UT Austin Perry-Castañeda Library. **Public Domain** (U.S. Government Work 17 U.S.C. § 105). Documents the exact canal-to-road transitions (Silom, Rama IV, Sathon) and pre-sprawl urban core.

All sources comply with the commercial-use requirements of [`CONTEXT.md`](../../../CONTEXT.md) and provide drop-in `sources.yaml` blocks and GDAL/QGIS digitization commands.
