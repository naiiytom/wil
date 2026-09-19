# Master Episode Sequence Manifest

Type: grilling
Status: resolved
Blocked by: none

## Question

How should the 5 Scene Sequences (Delta Exposure, Canal City, Subsidence Cutaway, Three Pressures, Drainage Capacity) be ordered, timed, and declared in a top-level episode specification (`episode.yaml`) to coordinate frame counts, scene transitions, and creator handoff for the full ~8-minute Episode 1?

## Answer

`episode.yaml` will orchestrate the full multi-sequence suite as a lean declarative conductor with dedicated CLI orchestration:

1. **`episode.yaml` Specification**:
   Lives at `packs/ep01-bangkok/episode.yaml`. Declares episode-level metadata and an ordered sequence registry:
   ```yaml
   title: "Why Bangkok Keeps Flooding"
   episode_id: "ep01-bangkok"
   target_runtime_seconds: 480.0
   sequences:
     - id: delta-exposure
       pack: 01-delta-exposure
       section: "0:00-0:35 Cold Open: Delta Elevation & Flood Exposure"
     - id: canal-city
       pack: 02-canal-city
       section: "0:35-3:00 Historical Water City & Urban Paving"
     - id: subsidence-cutaway
       pack: 03-subsidence-cutaway
       section: "3:00-4:30 Subterranean Clay Compaction"
     - id: three-pressures
       pack: 04-three-pressures
       section: "4:30-5:45 Rain, River, and Tide"
     - id: drainage-capacity
       pack: 05-drainage-capacity
       section: "5:45-7:35 Drainage Capacity Limits"
   ```
2. **Dedicated CLI Command (`render-episode`)**:
   Command: `world-in-layers render-episode <episode-pack> <output-dir> [--jobs N]`
   - Atomically pre-validates `episode.yaml`, all five child `sequence.yaml` files, shared geometries in `_shared/`, and all source references before writing any output.
   - Renders each sequence concurrently using Rayon into structured subdirectories (`<output-dir>/01-delta-exposure/`, etc.).
   - Emits a consolidated `episode-manifest.yaml` in `<output-dir>`.
3. **Atomic Failure Rollback**:
   On any error during the episode render, the temporary rendering directory is recursively unlinked (`fs::remove_dir_all`), leaving zero partial files or directories on disk.
4. **Assembly Handoff (`episode-manifest.yaml`)**:
   Provides an editorial roadmap for timeline assembly, mapping each sequence directory to its exact frame count, rendered duration, FPS, and script narrative section for drag-and-drop alignment against the narration audio track.
