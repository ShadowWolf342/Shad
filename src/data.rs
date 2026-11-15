//! BBS1 binary data-tree storage codec, reverse-engineered from BBS 2.4 / MC 26.3.
//! The film/replay classes persist ValueGroups through this generic tree format.

use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read, Write};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

const MAX_ITEMS: usize = 1_000_000;
const MAX_DEPTH: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub enum BbsValue {
    Map(BTreeMap<String, BbsValue>),
    List(Vec<BbsValue>),
    String(String),
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ByteArray(Vec<i8>),
    ShortArray(Vec<i16>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl BbsValue {
    pub fn map<K: Into<String>>(entries: impl IntoIterator<Item = (K, BbsValue)>) -> Self {
        Self::Map(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
    fn type_id(&self) -> u8 {
        match self {
            Self::Map(_) => 0, Self::List(_) => 1, Self::String(_) => 2,
            Self::Byte(_) => 3, Self::Short(_) => 4, Self::Int(_) => 5,
            Self::Float(_) => 6, Self::Long(_) => 7, Self::Double(_) => 8,
            Self::ByteArray(_) => 9, Self::ShortArray(_) => 10,
            Self::IntArray(_) => 11, Self::LongArray(_) => 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    Truncated,
    BadMagic,
    UnknownType(u8),
    InvalidKeyIndex(u32),
    InvalidUtf8,
    LengthLimit,
    TrailingData,
    DepthLimit,
    Compression,
}
impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CodecError {}

/// Decode a GZIP-wrapped BBS1 stream, as used by BBS FilmManager's `.dat` files.
pub fn decode_gzip_bbs1(bytes: &[u8]) -> Result<BbsValue, CodecError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).map_err(|_| CodecError::Compression)?;
    decode_bbs1(&raw)
}

/// Encode a GZIP-wrapped BBS1 stream, matching BBS CompressedDataStorage.
pub fn encode_gzip_bbs1(root: &BbsValue) -> Result<Vec<u8>, CodecError> {
    let raw = encode_bbs1(root)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw).map_err(|_| CodecError::Compression)?;
    encoder.finish().map_err(|_| CodecError::Compression)
}

/// Decode a complete uncompressed BBS1 storage stream.
pub fn decode_bbs1(bytes: &[u8]) -> Result<BbsValue, CodecError> {
    let mut r = Reader { c: Cursor::new(bytes), key_width: 0 };
    if r.take(4)? != b"BBS1" { return Err(CodecError::BadMagic); }
    let key_width = r.u8()?;
    if key_width > 2 { return Err(CodecError::UnknownType(key_width)); }
    r.key_width = key_width;
    let count = r.index(key_width)?;
    if count > MAX_ITEMS { return Err(CodecError::LengthLimit); }
    let mut keys = vec![String::new(); count];
    for _ in 0..count {
        let index = r.index(key_width)?;
        if index >= count { return Err(CodecError::InvalidKeyIndex(index as u32)); }
        keys[index] = r.modified_utf()?;
    }
    let root = read_value(&mut r, &keys, 0)?;
    if r.c.position() != bytes.len() as u64 { return Err(CodecError::TrailingData); }
    Ok(root)
}

/// Encode a BBS1 storage stream. Map keys are emitted in sorted order for stable output.
/// BBS itself uses a HashMap for key indexing, so byte-for-byte output order is not stable
/// across Java runs; the resulting stream is semantically equivalent.
pub fn encode_bbs1(root: &BbsValue) -> Result<Vec<u8>, CodecError> {
    let mut key_set = std::collections::BTreeSet::new();
    collect_keys(root, &mut key_set, 0)?;
    let keys: Vec<String> = key_set.into_iter().collect();
    let width = if keys.len() < 256 { 0 } else if keys.len() < 65_536 { 1 } else { 2 };
    let indexes: HashMap<&str, usize> = keys.iter().enumerate().map(|(i, k)| (k.as_str(), i)).collect();
    let mut out = Vec::new();
    out.extend_from_slice(b"BBS1");
    out.push(width as u8);
    write_index(&mut out, width, keys.len())?;
    for (i, key) in keys.iter().enumerate() {
        write_index(&mut out, width, i)?;
        write_modified_utf(&mut out, key)?;
    }
    write_value(&mut out, root, width, &indexes, 0)?;
    Ok(out)
}

fn collect_keys(v: &BbsValue, keys: &mut std::collections::BTreeSet<String>, depth: usize) -> Result<(), CodecError> {
    if depth > MAX_DEPTH { return Err(CodecError::DepthLimit); }
    match v {
        BbsValue::Map(m) => for (k, v) in m { keys.insert(k.clone()); collect_keys(v, keys, depth + 1)?; },
        BbsValue::List(a) => for v in a { collect_keys(v, keys, depth + 1)?; },
        _ => {}
    }
    Ok(())
}

fn write_value(out: &mut Vec<u8>, v: &BbsValue, width: usize, keys: &HashMap<&str, usize>, depth: usize) -> Result<(), CodecError> {
    if depth > MAX_DEPTH { return Err(CodecError::DepthLimit); }
    out.push(v.type_id());
    match v {
        BbsValue::Map(m) => {
            put_i32(out, checked_len(m.len())?);
            for (k, v) in m {
                write_index(out, width, *keys.get(k.as_str()).ok_or(CodecError::InvalidKeyIndex(u32::MAX))?)?;
                write_value(out, v, width, keys, depth + 1)?;
            }
        }
        BbsValue::List(a) => { put_i32(out, checked_len(a.len())?); for v in a { write_value(out, v, width, keys, depth + 1)?; } }
        BbsValue::String(s) => write_modified_utf(out, s)?,
        BbsValue::Byte(n) => out.push(*n as u8),
        BbsValue::Short(n) => out.extend_from_slice(&n.to_be_bytes()),
        BbsValue::Int(n) => put_i32(out, *n),
        BbsValue::Float(n) => put_i32(out, n.to_bits() as i32),
        BbsValue::Long(n) => out.extend_from_slice(&n.to_be_bytes()),
        BbsValue::Double(n) => out.extend_from_slice(&n.to_bits().to_be_bytes()),
        BbsValue::ByteArray(a) => { put_i32(out, checked_len(a.len())?); out.extend(a.iter().map(|n| *n as u8)); }
        BbsValue::ShortArray(a) => { put_i32(out, checked_len(a.len())?); for n in a { out.extend_from_slice(&n.to_be_bytes()); } }
        BbsValue::IntArray(a) => { put_i32(out, checked_len(a.len())?); for n in a { put_i32(out, *n); } }
        BbsValue::LongArray(a) => { put_i32(out, checked_len(a.len())?); for n in a { out.extend_from_slice(&n.to_be_bytes()); } }
    }
    Ok(())
}

fn read_value(r: &mut Reader<'_>, keys: &[String], depth: usize) -> Result<BbsValue, CodecError> {
    if depth > MAX_DEPTH { return Err(CodecError::DepthLimit); }
    Ok(match r.u8()? {
        0 => {
            let n = r.count()?; let mut m = BTreeMap::new();
            for _ in 0..n { let i = r.index(r.key_width)?; let k = keys.get(i).ok_or(CodecError::InvalidKeyIndex(i as u32))?.clone(); let v = read_value(r, keys, depth + 1)?; m.insert(k, v); }
            BbsValue::Map(m)
        }
        1 => { let n = r.count()?; let mut a = Vec::with_capacity(n); for _ in 0..n { a.push(read_value(r, keys, depth + 1)?); } BbsValue::List(a) }
        2 => BbsValue::String(r.modified_utf()?),
        3 => BbsValue::Byte(r.u8()? as i8),
        4 => BbsValue::Short(r.i16()?),
        5 => BbsValue::Int(r.i32()?),
        6 => BbsValue::Float(f32::from_bits(r.i32()? as u32)),
        7 => BbsValue::Long(r.i64()?),
        8 => BbsValue::Double(f64::from_bits(r.i64()? as u64)),
        9 => { let n = r.count()?; BbsValue::ByteArray(r.take(n)?.iter().map(|b| *b as i8).collect()) }
        10 => { let n = r.count()?; let mut a=Vec::with_capacity(n); for _ in 0..n {a.push(r.i16()?);} BbsValue::ShortArray(a) }
        11 => { let n = r.count()?; let mut a=Vec::with_capacity(n); for _ in 0..n {a.push(r.i32()?);} BbsValue::IntArray(a) }
        12 => { let n = r.count()?; let mut a=Vec::with_capacity(n); for _ in 0..n {a.push(r.i64()?);} BbsValue::LongArray(a) }
        id => return Err(CodecError::UnknownType(id)),
    })
}

struct Reader<'a> { c: Cursor<&'a [u8]>, key_width: u8 }
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<Vec<u8>, CodecError> {
        let start = self.c.position() as usize;
        let end = start.checked_add(n).ok_or(CodecError::LengthLimit)?;
        if end > self.c.get_ref().len() { return Err(CodecError::Truncated); }
        self.c.set_position(end as u64); Ok(self.c.get_ref()[start..end].to_vec())
    }
    fn u8(&mut self) -> Result<u8, CodecError> { Ok(self.take(1)?[0]) }
    fn i16(&mut self) -> Result<i16, CodecError> { Ok(i16::from_be_bytes(self.take(2)?.try_into().unwrap())) }
    fn i32(&mut self) -> Result<i32, CodecError> { Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap())) }
    fn i64(&mut self) -> Result<i64, CodecError> { Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap())) }
    fn count(&mut self) -> Result<usize, CodecError> { let n=self.i32()?; if n<0 || n as usize>MAX_ITEMS {Err(CodecError::LengthLimit)} else {Ok(n as usize)} }
    fn index(&mut self, width: u8) -> Result<usize, CodecError> { match width {0=>Ok(self.u8()? as usize),1=>Ok(self.i16()? as u16 as usize),2=>Ok(self.i32()? as u32 as usize),_=>Err(CodecError::UnknownType(width))} }
    fn modified_utf(&mut self) -> Result<String, CodecError> {
        let len = self.i16()? as u16 as usize;
        let b = self.take(len)?;
        let mut units = Vec::new(); let mut i=0;
        while i < b.len() {
            let x=b[i];
            if x & 0x80 == 0 { units.push(x as u16); i+=1; }
            else if x & 0xe0 == 0xc0 {
                if i+1>=b.len() || b[i+1]&0xc0!=0x80 {return Err(CodecError::InvalidUtf8);}
                units.push((((x&0x1f) as u16)<<6)|((b[i+1]&0x3f) as u16)); i+=2;
            } else if x & 0xf0 == 0xe0 {
                if i+2>=b.len() || b[i+1]&0xc0!=0x80 || b[i+2]&0xc0!=0x80 {return Err(CodecError::InvalidUtf8);}
                units.push((((x&0x0f) as u16)<<12)|(((b[i+1]&0x3f) as u16)<<6)|((b[i+2]&0x3f) as u16)); i+=3;
            } else { return Err(CodecError::InvalidUtf8); }
        }
        String::from_utf16(&units).map_err(|_| CodecError::InvalidUtf8)
    }
}

fn write_modified_utf(out: &mut Vec<u8>, s: &str) -> Result<(), CodecError> {
    let mut bytes=Vec::new();
    for u in s.encode_utf16() {
        if u==0 { bytes.extend_from_slice(&[0xc0,0x80]); }
        else if u<=0x7f { bytes.push(u as u8); }
        else if u<=0x7ff { bytes.push(0xc0|((u>>6) as u8)); bytes.push(0x80|(u as u8&0x3f)); }
        else { bytes.push(0xe0|((u>>12) as u8)); bytes.push(0x80|((u>>6) as u8&0x3f)); bytes.push(0x80|(u as u8&0x3f)); }
    }
    if bytes.len()>u16::MAX as usize {return Err(CodecError::LengthLimit);}
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes()); out.extend(bytes); Ok(())
}
fn checked_len(n: usize) -> Result<i32, CodecError> { if n>MAX_ITEMS {Err(CodecError::LengthLimit)} else {Ok(n as i32)} }
fn write_index(out: &mut Vec<u8>, width: usize, n: usize) -> Result<(), CodecError> {
    match width {0 if n<=u8::MAX as usize=>out.push(n as u8),1 if n<=u16::MAX as usize=>out.extend_from_slice(&(n as u16).to_be_bytes()),2 if n<=i32::MAX as usize=>put_i32(out,n as i32),_=>return Err(CodecError::LengthLimit)} Ok(())
}
fn put_i32(out: &mut Vec<u8>, n: i32) { out.extend_from_slice(&n.to_be_bytes()); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_reference_bbs1_golden_fixture() {
        // Derived from DataStorage.writeToStream + MapType/IntType bytecode:
        // {"a": 42}; dictionary uses BYTE indexes, map tag=0, int tag=5.
        let fixture = b"BBS1\x00\x01\x00\x00\x01a\x00\x00\x00\x01\x00\x05\x00\x00\x00*";
        let value = BbsValue::map([("a", BbsValue::Int(42))]);
        assert_eq!(decode_bbs1(fixture).unwrap(), value);
        assert_eq!(encode_bbs1(&value).unwrap(), fixture);
    }
    #[test]
    fn nested_film_like_tree_round_trips_all_value_kinds() {
        let mut root = BTreeMap::new();
        root.insert("name".into(), BbsValue::String("Take \0 🎬".into()));
        root.insert("fps".into(), BbsValue::Float(24.0));
        root.insert("format".into(), BbsValue::Int(2));
        root.insert("camera".into(), BbsValue::List(vec![
            BbsValue::Map(BTreeMap::from([("tick".into(), BbsValue::Int(0)), ("yaw".into(), BbsValue::Double(90.25))]))
        ]));
        root.insert("raw".into(), BbsValue::ByteArray(vec![-1, 0, 1]));
        root.insert("samples".into(), BbsValue::LongArray(vec![1, -2, 3]));
        let value = BbsValue::Map(root);
        let encoded = encode_bbs1(&value).unwrap();
        assert_eq!(decode_bbs1(&encoded).unwrap(), value);
    }
    #[test]
    fn gzip_storage_round_trips_like_film_manager() {
        let value = BbsValue::map([("bbs_format", BbsValue::Int(4)), ("description", BbsValue::String("compressed film".into()))]);
        let bytes = encode_gzip_bbs1(&value).unwrap();
        assert_eq!(decode_gzip_bbs1(&bytes).unwrap(), value);
    }
    #[test]
    fn rejects_bad_header_and_trailing_bytes() {
        assert_eq!(decode_bbs1(b"NOPE"), Err(CodecError::BadMagic));
        let mut bytes=encode_bbs1(&BbsValue::Int(1)).unwrap(); bytes.push(0);
        assert_eq!(decode_bbs1(&bytes), Err(CodecError::TrailingData));
    }
}
