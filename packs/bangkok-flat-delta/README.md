# Bangkok's Flat Delta

This pack provides a 12-second, 4K/30 fps explanatory sequence about rainfall overload, river back-pressure, and tide-blocked outflow. Its sequence geography is local WGS 84 GeoJSON with provenance in `data/raw/README.md`; the rainfall treatment and timing are illustrative.

Run it from the repository root:

```powershell
cargo run -- render .\packs\bangkok-flat-delta .\test.png
```

The Scene retains OpenStreetMap attribution and identifies NASA SRTM as the elevation baseline. See `sources.yaml` before publishing a derivative.

Render the sequence from the repository root:

```powershell
cargo run -- render-sequence .\packs\bangkok-flat-delta .\target\bangkok-frames
```

The renderer performs no remote fetches. See `sources.yaml` and `data/raw/README.md` before publishing a derivative.
