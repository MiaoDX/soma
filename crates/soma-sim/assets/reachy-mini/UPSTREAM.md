# Reachy Mini MuJoCo assets

Source: `pollen-robotics/reachy_mini`

- Commit: `20bc9eedc81ddc552235d222ca7e39205b2c2481`
- Upstream path: `src/reachy_mini/descriptions/reachy_mini/mjcf`
- License: Apache-2.0; see `LICENSE`
- Imported slice: `scene.xml`, `reachy_mini.xml`, and only the STL files
  referenced by `reachy_mini.xml`

The analytical showcase additionally pins:

- Reachy Mini release: `v1.9.0` (`b7e686d994a178353ebf81ea935de82ce65af733`)
- Upstream path: `src/reachy_mini/assets/kinematics_data.json`
- Upstream SHA-256: `b07324cfe46515d7b3bd5be416f5f2ed87811ac1fcdb0e6ad841119e4ed9ab3b`
- Vendored SHA-256: `0c21a2d30b026bc90b11e0efa60fb5b8880d1e666183797791a32104e692ec07`
- Binding: `reachy-mini-rust-kinematics==1.0.3`

The JSON values are unmodified; the vendored text adds one final newline. It is
consumed directly without the full `reachy_mini` Python package.

The files are unmodified. Unreferenced example-scene textures, backup XML,
FreeCAD source parts, and alternative fine collision meshes are excluded.
