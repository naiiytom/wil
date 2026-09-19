# Parallel Frame Rendering Architecture

Type: grilling
Status: resolved
Blocked by: none

## Question

How should `render-sequence` parallelize 4K frame generation across CPU threads (e.g. via `rayon` or thread pools) while guaranteeing deterministic frame naming, bounded memory allocation under concurrent `resvg` pixmaps, and atomic cleanup if any frame fails?

## Answer

`render-sequence` will execute frame synthesis concurrently using Rayon and a shared font database:

1. **Parallel Engine**: Add `rayon = "1.10"` to `Cargo.toml`. Parallelize frame rendering via `(0..frame_count).into_par_iter().try_for_each(|frame| ...)`. This leverages work-stealing across CPU cores to balance frames of varying visual complexity.
2. **Shared Font Database**: Initialize `resvg::usvg::fontdb::Database` once before starting the frame iteration. Pass a shared reference (`&Database` or `Arc<Database>`) into the SVG rendering pipeline, eliminating redundant per-frame system font filesystem scans.
3. **Thread Pool Configuration**: Support an optional `--jobs <N>` / `-j <N>` argument in the CLI (`render-sequence <pack> <output> [--jobs N]`). When specified, construct a local scoped `rayon::ThreadPool`; when omitted, default to Rayon's global pool (saturating available logical CPU cores).
4. **Deterministic Output & Atomic Failure**: Frames are written directly to `temporary.join(format!("frame-{frame:06}.png"))`. If any frame encounters an error, `try_for_each` halts further processing, waits for active tasks to unwind, completely removes `temporary` via `fs::remove_dir_all`, and propagates the error, guaranteeing all-or-nothing determinism.
