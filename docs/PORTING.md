# Porting plan

## Target

- Elyhorizon release: `elyhorizon-v0.2.3-beta.1`
- Exact Horizon source commit: `faf034f0b769d1967316fb52a381799413618c6a`
- Minecraft Java Edition 26.3
- Native delivery: `.hnmod` source-mixin bundle
- Runtime implementation: Rust only; no JVM, Java source, Fabric Loader or Fabric API.

Horizon documents `.hnmod` as native source that is compiled into a custom game executable, not loaded as a runtime plugin. The target release also has a trait-based native `Hooks` API and a separate `horizon-mixins` source hook mechanism. The adapter must be verified against this exact commit; newer templates are not assumed compatible.

## BBS reference inventory

The uploaded BBS 2.4 / Minecraft 26.3 JAR contains 1,747 classes. Package counts:

| Package | Classes | Port focus |
|---|---:|---|
| UI | 700 | Film editor, timeline, forms, panels and widgets |
| Utils | 178 | Keyframes, interpolation, serialization and helpers |
| Cubic | 124 | Model hierarchy, geometry and animation |
| Forms | 105 | Typed property editors |
| Mixin | 77 | Map game behavior to native hooks |
| Math | 76 | Vectors, matrices and curves |
| Graphics | 72 | Viewport, rendering and post effects |
| Settings | 58 | Film and replay properties |
| Particles | 58 | Particle data and dispatch |
| Camera | 51 | Camera paths, clips and modifiers |
| Data | 35 | BBS data tree and persistence |
| Film | 33 | Films, replays, markers and recording |
| Actions | 27 | Timeline action clips |
| Client | 25 | Client lifecycle and services |
| Audio | 19 | Audio clips and capture |
| BOBJ | 15 | BBS model format |
| VOX | 12 | MagicaVoxel model format |
| Events | 11 | Event dispatch |
| Importers | 11 | Asset import |
| OBJ | 10 | Wavefront model format |
| Network | 9 | BBS network messages |
| Resources | 9 | Resource management |
| Root | 8 | Main package classes |
| Localization | 8 | Translations |
| Entity | 4 | Entity helpers |
| Selectors | 4 | Selection helpers |
| Blocks | 3 | Block integration |
| Items | 3 | Item integration |
| Morphing | 2 | Morphing integration |

BBS's Java UI framework cannot be transplanted. Its editor workflows and data model must be recreated using Horizon's native Rust-facing facilities.

## Milestones

1. Core film/replay data, time, keyframe channels, looping sections, interpolation and transforms.
2. BBS serialization compatibility, including legacy formats and compressed storage.
3. Native engine integration: registration, frame/tick lifecycle, input, config and persistence.
4. Camera clips, animation, model hierarchy and Molang-compatible evaluation.
5. Editor UI: dashboard, film timeline, clip panels, replay list, forms, keyframe editing, undo/redo and viewport gizmos.
6. Action clips, recording, entity state capture and playback.
7. Rendering, overlays, filters, subtitles, image/audio clips and importers.
8. Round-trip fixtures, packaged `.hnmod`, and an in-game smoke test on the pinned Elyhorizon build.

## Current implementation

The initial Rust core contains film/replay/marker containers, vector and transform primitives, BBS-style keyframe section timing, repeat-to-source mapping, ghost-tick queries, and basic playback state. It does not yet contain a BBS file parser, editor UI, camera hooks, engine integration or a packaged `.hnmod`.
