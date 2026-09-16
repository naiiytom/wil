# Bangkok's Flat Delta

This is the first runnable, deliberately simplified explanatory Scene. Its hand-authored geometry is not a GIS-accurate map; replace it with locally curated GeoJSON when geographic precision is required.

Run it from the repository root:

```powershell
cargo run -- render .\packs\bangkok-flat-delta .\test.png
```

The Scene retains OpenStreetMap attribution and identifies NASA SRTM as the elevation baseline. See `sources.yaml` before publishing a derivative.
