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


## BBS 2.4 film/replay storage findings (from reference bytecode)

The BBS 2.4 film manager creates a CompressedDataStorage and uses the .dat extension. That storage wraps a DataGzipStorage around a DataFileStorage; the latter delegates to the generic BBS DataStorage codec. Therefore the on-disk file is GZIP-compressed BBS1 data, not plain JSON.

The uncompressed stream layout verified from DataStorage.readFromStream/writeToStream and the value type implementations is:

1. Four-byte ASCII magic BBS1.
2. One-byte key-index width: 0 = unsigned byte, 1 = unsigned short, 2 = signed Java int (nonnegative indexes).
3. Key dictionary count encoded using that width, followed by (index using that width, Java modified-UTF string) entries. Dictionary entry order is not guaranteed by Java's HashMap; entries must be placed by their explicit index.
4. One recursively encoded root value. Type IDs are: map=0, list=1, string=2, byte=3, short=4, int=5, float=6, long=7, double=8, byte[]=9, short[]=10, int[]=11, long[]=12. Map/list and array lengths are big-endian 32-bit integers. Map keys are dictionary indexes. Numeric primitives are big-endian; strings use Java DataOutput modified UTF-8.

The dictionary width selection in BBS is byte for fewer than 256 distinct keys, short for fewer than 65,536, otherwise int. Map and list sizes are always 32-bit integers. BBS rejects non-BBS1 headers and trailing bytes.

The film root's registered value names observed in Film() are: bbs_format (default 4), camera, replays, markers, replay_categories, inventory, hp, hunger, xp_level, xp_progress, mob_recording_radius, relative_coordinates, description, created_at, and time_spent_active. Replay groups register form, anchor, props, sections, actions, animation, layers, enabled, enabled_start, enabled_end, category, label, name_tag, shadow, shadow_size, shadow_follow, shadow_offset, looping, actor, fp, relative, relative_offset, axes_preview, and axes_preview_bone. Each keyframe section stores start, duration, repeats, accumulate, and channels. These are the ValueGroup registration names, not a claim that every value is always present in a saved file.

## Implemented serialization slice

src/data.rs implements the BBS1 typed data tree, dictionary index widths, Java modified UTF-8, and GZIP-wrapped BBS storage. The codec preserves the tree values used by films/replays, including maps, lists, all scalar numeric types, and primitive arrays. Its stable Rust writer sorts map keys; BBS's Java writer uses HashMap iteration so byte-for-byte ordering is not a compatibility requirement. Semantic tree equality is the round-trip criterion.

Tests include a small BBS1 byte fixture derived directly from the reference serialization bytecode, a nested film-like tree round trip covering all value kinds, malformed-header/trailing-data checks, and GZIP round-trip coverage. These are codec-level compatibility tests; they are not yet real film files exported by BBS. A full typed Film/Replay importer, legacy migration, keyframe-channel factory decoding, and import/export against actual user-created .dat fixtures remain outstanding.
