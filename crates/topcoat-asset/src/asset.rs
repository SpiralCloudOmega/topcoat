use std::{
    io::Read,
    path::{Path, PathBuf},
};

use memchr::memmem;
use serde::{Deserialize, Serialize};

use crate::hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetId(u64);

impl AssetId {
    pub const fn from_path(path: &str) -> Self {
        Self(hash::fnv1a(path.as_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    id: AssetId,
    path: String,
    crate_name: String,
    manifest_dir: String,
    source_file: String,
}

pub const ENCODED_ASSET_SIZE: usize = 2048;

impl Asset {
    pub const fn encode(
        id: AssetId,
        path: &str,
        crate_name: &str,
        manifest_dir: &str,
        source_file: &str,
    ) -> [u8; ENCODED_ASSET_SIZE] {
        let mut out = [0u8; ENCODED_ASSET_SIZE];
        let mut w = ConstWriter {
            buf: &mut out,
            pos: 0,
        };
        w.write_bytes(&asset_prefix());
        w.write_bytes(&id.0.to_le_bytes());
        w.write_str(path);
        w.write_str(crate_name);
        w.write_str(manifest_dir);
        w.write_str(source_file);
        out
    }

    pub fn decode(buffer: &[u8]) -> Option<Self> {
        let mut cur = buffer.get(asset_prefix().len()..)?;

        let mut id_buf = [0u8; 8];
        cur.read_exact(&mut id_buf).ok()?;
        let id = AssetId(u64::from_le_bytes(id_buf));

        Some(Self {
            id,
            path: read_str(&mut cur)?,
            crate_name: read_str(&mut cur)?,
            manifest_dir: read_str(&mut cur)?,
            source_file: read_str(&mut cur)?,
        })
    }

    pub fn find_in_binary(binary: &[u8]) -> Vec<Self> {
        let prefix = asset_prefix();
        let finder = memmem::Finder::new(&prefix);
        finder
            .find_iter(binary)
            .filter_map(|index| Self::decode(&binary[index..]))
            .collect()
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    /// Resolve the asset path relative to the source file it was declared in.
    ///
    /// `file!()` is relative to the crate's manifest dir for first-party
    /// sources but absolute for dependencies built from the cargo cache;
    /// `Path::join` collapses both cases since an absolute right-hand side
    /// replaces the base.
    pub fn resolved_path(&self) -> PathBuf {
        let source = Path::new(&self.manifest_dir).join(&self.source_file);
        let parent = source.parent().unwrap_or(Path::new(""));
        parent.join(&self.path)
    }
}

struct ConstWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl ConstWriter<'_> {
    const fn write_bytes(&mut self, bytes: &[u8]) {
        let mut i = 0;
        while i < bytes.len() {
            self.buf[self.pos] = bytes[i];
            self.pos += 1;
            i += 1;
        }
    }

    const fn write_str(&mut self, s: &str) {
        self.write_bytes(&(s.len() as u16).to_le_bytes());
        self.write_bytes(s.as_bytes());
    }
}

fn read_str(cur: &mut &[u8]) -> Option<String> {
    let mut len_buf = [0u8; 2];
    cur.read_exact(&mut len_buf).ok()?;
    let len = u16::from_le_bytes(len_buf) as usize;
    let bytes = cur.get(..len)?;
    let s = std::str::from_utf8(bytes).ok()?.to_owned();
    *cur = &cur[len..];
    Some(s)
}

#[macro_export]
macro_rules! asset {
    ($path:expr) => {{
        const PATH: &str = $path;
        const ID: $crate::AssetId = $crate::AssetId::from_path(PATH);
        const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
        const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
        const SOURCE_FILE: &str = file!();

        #[used]
        pub static ENCODED_ASSET: [u8; $crate::ENCODED_ASSET_SIZE] =
            $crate::Asset::encode(ID, PATH, CRATE_NAME, MANIFEST_DIR, SOURCE_FILE);

        ID
    }};
}

const PREFIX_KEY: u8 = 0xA7;

// "TOPCOAT_ASSET" XOR'd byte-by-byte with PREFIX_KEY. Storing the scrambled
// form means the literal marker only appears in binaries that actually carry
// an asset (where `asset_prefix` unscrambles it into the embedded payload),
// not in every binary that just links this crate.
const SCRAMBLED_PREFIX: [u8; 13] = [
    b'T' ^ PREFIX_KEY,
    b'O' ^ PREFIX_KEY,
    b'P' ^ PREFIX_KEY,
    b'C' ^ PREFIX_KEY,
    b'O' ^ PREFIX_KEY,
    b'A' ^ PREFIX_KEY,
    b'T' ^ PREFIX_KEY,
    b'_' ^ PREFIX_KEY,
    b'A' ^ PREFIX_KEY,
    b'S' ^ PREFIX_KEY,
    b'S' ^ PREFIX_KEY,
    b'E' ^ PREFIX_KEY,
    b'T' ^ PREFIX_KEY,
];

const fn asset_prefix() -> [u8; 13] {
    let mut out = [0u8; 13];
    let mut i = 0;
    while i < SCRAMBLED_PREFIX.len() {
        out[i] = SCRAMBLED_PREFIX[i] ^ PREFIX_KEY;
        i += 1;
    }
    out
}
