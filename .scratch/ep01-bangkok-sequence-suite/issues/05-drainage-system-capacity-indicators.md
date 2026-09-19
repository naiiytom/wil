# Drainage System Capacity Indicators

Type: grilling
Status: resolved
Blocked by: 02

## Question

How should pump station capacities, tidal back-pressure thresholds, and canal overflow points be visually represented and animated within the paper diorama visual language using existing layer and label keyframes?

## Answer

Sequence 05 (`packs/ep01-bangkok/05-drainage-capacity/`) will explain drainage bottlenecks and mechanical thresholds across three phased Scenes using existing layer and label keyframe capabilities:

1. **Capacity Saturation & Network Transition**:
   - Macro canal networks transition from calm blue (`#0284c7`, nominal flow) to amber warning (`#f59e0b`), and finally to crimson overload (`#dc2626`) by cross-fading duplicate layer geometries via opacity keyframes.
   - Status badge labels are keyframed at critical geographic nodes (e.g. `PHRA KHANONG PUMP STATION: 100% CAPACITY`, `FLOODGATES CLOSED: GRAVITY DISCHARGE 0 m³/s`, `CANAL RETENTION SURPASSED`).
2. **Tidal Collision & Floodgate Barriers**:
   - Rising Gulf tide is visualized as opposing crimson lines surging upstream along the Chao Phraya against the downstream canal discharge.
   - Floodgate closures are rendered as high-contrast red barrier lines (`kind: line`, `#ef4444`, `stroke_width: 16`) sealing canal estuaries, illustrating that gates must close to prevent river intrusion, which halts gravity outflow.
3. **Spatial Framing**:
   - The scene maintains the macro Greater Bangkok drainage basin framing (Bang Sue to the Gulf mouth at Samut Prakan), preserving geographic continuity with earlier sequences in the episode.
4. **Explanatory Phasing (12 seconds, 30 fps)**:
   - `nominal-pumping` (`0.0–4.0s`): Normal rainfall, gravity flow active, pump stations operating within design limits (blue network cues).
   - `tidal-surge-gate-closure` (`4.0–8.0s`): Tide rises in the Gulf; sea level exceeds canal elevation; river floodgates lock shut; gravity outflow drops to zero.
   - `system-capacity-overload` (`8.0–12.0s`): Sustained monsoon precipitation (100+ mm/hr) overwhelms pump station capacity; canal levels swell and spill into urban street grid polygons; concluding with the Attribution Card: *"Geographic features are source-backed; timing is illustrative."*
