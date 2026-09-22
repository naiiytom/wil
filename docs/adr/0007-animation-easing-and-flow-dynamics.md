# Animation Easing and Flow Dynamics

World in Layers will support non-linear keyframe easing (`linear`, `ease-in`, `ease-out`, `ease-in-out`, `cubic-in`, `cubic-out`, `cubic-in-out`) and animatable `stroke_dashoffset` / `stroke_dasharray` flow dynamics directly in the declarative Scene Sequence schema. This brings tactile, organic motion deceleration and fluid directional flow to geographic systems (rivers, tidal pressure vectors, drainage paths, Map Transitions) while maintaining strict offline determinism, backwards compatibility, and zero new runtime dependencies.

## Schema Additions and Defaults

1. **`NumberKeyframe`**:
   - `easing: Option<Easing>` (optional, defaults to `linear` / `None`).
   - Supported easing variants: `linear`, `ease-in`, `ease-out`, `ease-in-out`, `cubic-in`, `cubic-out`, `cubic-in-out`.
   - Applied to the interval starting at the given keyframe when interpolating toward the subsequent keyframe.

2. **`Layer`**:
   - `stroke_dasharray: Option<String>` (optional, defaults to `None`).
   - `stroke_dashoffset: f32` (optional, defaults to `0.0`).
   - Applied to both `line` (<polyline>) and `polygon` (<polygon>) elements, as well as their drop shadows when lifted.

3. **`LayerAnimation`**:
   - `stroke_dashoffset: Vec<NumberKeyframe>` (optional, defaults to `[]`).
   - Validated for non-decreasing monotonic timestamps within `[0, duration_seconds]`.
   - Evaluated per frame to drive continuous directional flow along geographic lines and boundaries.
