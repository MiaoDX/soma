# Rust ONNX Runtime Provisioning

Soma's Rust policy worker dynamically loads the CPU ONNX Runtime shared
library. The native runtime is a deployment prerequisite and is deliberately
not committed to this repository or bundled into every daemon release.

The supported provisioning path pins Microsoft ONNX Runtime `1.28.0`, selects
the host archive for Linux `x86_64` or `aarch64`, verifies the release archive
with the SHA-256 embedded in [`scripts/provision-onnx-runtime`](../../scripts/provision-onnx-runtime),
and installs it under the user cache by default. Set `SOMA_ORT_PREFIX` to use a
machine-managed location instead.

```bash
scripts/provision-onnx-runtime --install
eval "$(scripts/provision-onnx-runtime --print-env)"
scripts/provision-onnx-runtime --check
```

The Rust worker should receive `ORT_DYLIB_PATH` in its service environment.
Do not put the path in source or rely on an unpinned system library. A failed,
partial, or checksum-mismatched prefix is refused rather than overwritten;
remove that explicitly after inspecting it, then rerun provisioning.

The Rust wrapper and native runtime remain separate pins: the workspace pins
the `ort` crate, while this script pins the native ONNX Runtime release. The
runtime must be initialized and warmed up before the periodic `robot-rt` path;
`robot-rt` itself must never load ONNX or perform blocking I/O.
