use std::path::{Path, PathBuf};

use const_serialize::{
    ConstReadBuffer, ConstStr, ConstVec, SerializeConst, deserialize_const, serialize_const,
};
use memchr::memmem;
use serde::{Deserialize, Serialize};

use crate::hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeConst, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetId(u64);

impl AssetId {
    pub const fn from_path(path: &str) -> Self {
        Self(hash::fnv1a(path.as_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq, SerializeConst)]
pub struct Asset {
    id: AssetId,
    path: ConstStr,
    crate_name: ConstStr,
    manifest_dir: ConstStr,
    source_file: ConstStr,
}

impl Asset {
    pub const fn new(
        id: AssetId,
        path: &'static str,
        crate_name: &'static str,
        manifest_dir: &'static str,
        source_file: &'static str,
    ) -> Self {
        Self {
            id,
            path: ConstStr::new(path),
            crate_name: ConstStr::new(crate_name),
            manifest_dir: ConstStr::new(manifest_dir),
            source_file: ConstStr::new(source_file),
        }
    }

    pub const fn encode(&self) -> [u8; 2048] {
        let mut buffer = ConstVec::<u8, 2048>::new_with_max_size();
        buffer = buffer.extend(&asset_prefix());
        buffer = serialize_const(self, buffer);

        let mut out = [0u8; 2048];
        let src = buffer.as_ref();
        let mut i = 0;
        while i < buffer.len() {
            out[i] = src[i];
            i += 1;
        }
        out
    }

    pub fn decode(buffer: &[u8]) -> Option<Self> {
        let buffer = ConstReadBuffer::new(buffer.get(SCRAMBLED_PREFIX.len()..)?);
        deserialize_const!(Asset, buffer).map(|(_, asset)| asset)
    }

    pub fn find_in_binary(binary: &[u8]) -> Vec<Self> {
        let prefix = asset_prefix();
        let finder = memmem::Finder::new(&prefix);
        finder
            .find_iter(binary)
            .filter_map(|index| Self::decode(&binary[index..]))
            .collect()
    }

    pub const fn id(&self) -> AssetId {
        self.id
    }

    pub const fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Resolve the asset path relative to the source file it was declared in.
    ///
    /// `file!()` is relative to the crate's manifest dir for first-party
    /// sources but absolute for dependencies built from the cargo cache;
    /// `Path::join` collapses both cases since an absolute right-hand side
    /// replaces the base.
    pub fn resolved_path(&self) -> PathBuf {
        let source = Path::new(self.manifest_dir.as_str()).join(self.source_file.as_str());
        let parent = source.parent().unwrap_or(Path::new(""));
        parent.join(self.path.as_str())
    }
}

#[macro_export]
macro_rules! asset {
    ($path:expr) => {{
        const PATH: &str = $path;
        const ID: ::topcoat::asset::AssetId = ::topcoat::asset::AssetId::from_path(PATH);
        const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
        const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
        const SOURCE_FILE: &str = file!();

        #[used]
        pub static ENCODED_ASSET: [u8; 2048] =
            const { $crate::Asset::new(ID, PATH, CRATE_NAME, MANIFEST_DIR, SOURCE_FILE).encode() };

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
