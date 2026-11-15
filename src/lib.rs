//! BBS native Rust port: format and timeline primitives.

pub mod data;
pub mod film;
pub mod math;
pub mod timeline;

pub use data::{BbsValue, CodecError, decode_bbs1, decode_gzip_bbs1, encode_bbs1, encode_gzip_bbs1};
pub use film::{Film, Marker, Replay};
pub use math::{Transform, Vec3};
pub use timeline::{Keyframe, KeyframeSection, Playback, PlaybackState};
