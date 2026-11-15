# BBS for Elyhorizon — native Rust port

A native Rust recreation of the BBS animation studio for Elyhorizon / Horizon 26.3. The Java/Fabric JARs are used only as behavioral and file-format references; the implementation in this repository is Rust.

## Target

- Elyhorizon release: `elyhorizon-v0.2.3-beta.1`
- Horizon source revision: `faf034f0b769d1967316fb52a381799413618c6a`
- Game: Minecraft Java Edition 26.3
- Native package: `.hnmod` (Horizon source-mixin bundle)
- JVM, Java source, Fabric Loader and Fabric API are not runtime dependencies.

## Status

This is the initial project scaffold, not a completed port. The first implementation slice mirrors BBS film/replay data structures and the looping keyframe-section rules. Native UI, rendering, camera, model, audio, import/export and recording systems are planned as separate milestones.

See [docs/PORTING.md](docs/PORTING.md) for the subsystem map and validation plan.

## Build

The standalone Rust core is intentionally dependency-light:

```sh
cargo test
cargo fmt --all --check
```

Packaging and compiling the `.hnmod` against the pinned Horizon source will be enabled as the native hook integration is implemented. The native bundle is compiled into a Horizon executable; it is not a runtime-loaded Java/Fabric mod.

## Reference material

The uploaded BBS JARs are not redistributed here. Their metadata, public class signatures, assets and behavior are used as references while the Rust implementation is written. BBS 2.4 metadata identifies the project license as MIT; preserve upstream attribution and license notices if any upstream code or assets are ever redistributed.
