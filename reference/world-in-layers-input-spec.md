# World in Layers — Input Specifications

Extracted from the FSD (sections 26–34).

# 26. Geographic Data System

Required input formats eventually:

```text
GeoJSON
TopoJSON
CSV
custom JSON
SVG
```

Later:

```text
Shapefile preprocessing
DEM raster
GeoTIFF
OpenStreetMap extracts
```

---

# 27. Dataset Preprocessing

The runtime renderer should avoid performing unnecessary heavy preprocessing repeatedly.

Dataset workflow:

```text
RAW DATA
   ↓
WIL DATA PREPROCESSOR
   ↓
Normalized coordinates
   ↓
Simplified geometry
   ↓
Topology
   ↓
Metadata
   ↓
.wildata
```

Proposed internal format:

```text
.wildata
```

---

# 28. Procedural Generation

The engine should eventually support deterministic procedural components.

Initial examples:

```text
BuildingCluster
RainSystem
RoadDecoration
TreeCluster
RouteParticles
```

Each procedural system must accept a seed.

Example:

```yaml
building_cluster:
  seed: 4928
  density: 0.72
```

---

# 29. Building Generation

Inputs:

```text
polygon
density
height distribution
minimum spacing
style
seed
```

Output:

```text
building instances
```

Buildings should be GPU-instanced where practical.

---

# 30. Rain System

Inputs:

```text
region
intensity
velocity
direction
seed
```

Output:

```text
rain particle instances
```

---

# 31. Flood System

MVP flood animation does not require full hydrodynamic simulation.

Initial model:

```text
Flood polygon
+
level
+
spread mask
+
timeline
```

Later versions may support elevation-based spreading.

---

# 32. Asset Registry

All reusable components should be discoverable through a central registry.

Example:

```text
geography.river

geography.terrain

city.building.lowrise

effects.raindrop

infrastructure.pump

overlay.flow-arrow
```

Registry metadata:

```text
component ID
version
category
properties
states
animations
thumbnail
documentation
```

---

# 33. Scene Serialization

Scenes must be serializable.

Primary format:

```text
JSON
```

Optional authoring format:

```text
YAML
```

JSON becomes authoritative at runtime.

---

# 34. Scene Schema

Proposed simplified example:

```json
{
  \"id\": \"bangkok-pressure\",
  \"duration\": 12,
  \"camera\": {
    \"preset\": \"high-angle\"
  },
  \"nodes\": [
    {
      \"id\": \"terrain\",
      \"component\": \"geography.terrain\",
      \"dataset\": \"bangkok-elevation\"
    },
    {
      \"id\": \"river\",
      \"component\": \"geography.river\",
      \"dataset\": \"chao-phraya\"
    }
  ]
}
```

Schema should be versioned.

Example:

```text
schemaVersion: 1
```

---
