# Subsidence Cross-Section Cutaway Design

Type: prototype
Status: resolved
Blocked by: 02

## Question

How should the subterranean geological cutaway (Bangkok soft clay compaction, deep aquifer depletion, and sinking structural piles) be composed and styled using 2D canvas paper polygons and keyframed explanatory timing?

## Answer

Subterranean geological cutaways will be represented using standard canvas-space polygon and line layers without requiring new engine schemas.

A working prototype was built and verified in [`packs/prototypes/subsidence-cutaway/`](../../../packs/prototypes/subsidence-cutaway/) and rendered to `target/subsidence-cutaway-frames`:

1. **Layer Geometry & Paper Styling**:
   - Geologic strata (sand aquifer `#334155`, stiff clay `#475569`, soft marine clay `#78716c`, and topsoil `#a8a29e`) are modeled as canvas-space polygons with distinct paper depth (`lift: 4` to `14`).
   - Structural piles and extraction wells are composed as styled `kind: line` elements.
   - An immutable 1960 historical datum line (`#ef4444`) and river high-tide level (`#2563eb`) establish the visual baseline.
2. **Explanatory Timing & Mechanics**:
   - Scene 1 (`0–4s`): Equilibrium baseline with active deep well pumping.
   - Scene 2 (`4–8s`): Keyframed vertical compaction (`translate.y: 140px`) on the soft clay layer, ground surface, and shallow-foundation building.
   - The deep-piled high-rise remains stationary, visibly exposing foundation piles and showing the sunken ground surface dropping below river high-tide level.
3. **Production Implementation**:
   - The sequence pack for Section 4 of Episode 1 will be authored under `packs/ep01-bangkok/03-subsidence-cutaway/` using this exact model.
