use std::path::{Path, PathBuf};

use memchr::memmem;
use serde::{Deserialize, Serialize};

use crate::{
    cursor::{ConstReader, ConstWriter},
    hash,
};

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
        let mut w = ConstWriter::new(&mut out);
        w.write_bytes(&asset_prefix());
        w.write_u64_le(id.0);
        w.write_str(path);
        w.write_str(crate_name);
        w.write_str(manifest_dir);
        w.write_str(source_file);
        out
    }

    pub fn decode(buffer: &[u8]) -> Option<Self> {
        let mut r = ConstReader::new(buffer);
        r.skip(asset_prefix().len())?;
        Some(Self {
            id: AssetId(r.read_u64_le()?),
            path: r.read_str()?.to_owned(),
            crate_name: r.read_str()?.to_owned(),
            manifest_dir: r.read_str()?.to_owned(),
            source_file: r.read_str()?.to_owned(),
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
