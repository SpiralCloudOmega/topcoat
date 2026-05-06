use const_serialize::{
    ConstReadBuffer, ConstStr, ConstVec, SerializeConst, deserialize_const, serialize_const,
};
use memchr::memmem;

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

#[derive(Debug, Clone, PartialEq, SerializeConst)]
pub struct Asset {
    path: ConstStr,
}

impl Asset {
    pub const fn new(path: &str) -> Self {
        Self {
            path: ConstStr::new(path),
        }
    }

    pub const fn path(&self) -> &str {
        self.path.as_str()
    }
}

#[macro_export]
macro_rules! asset {
    () => {};
}

pub const KEK: &[u8] = {
    #[used]
    pub static ASSET: [u8; 1024] = const {
        let mut buffer = ConstVec::new();
        buffer = buffer.extend(&asset_prefix());
        buffer = serialize_const(&Asset::new("./kek.png"), buffer);

        let mut out = [0u8; 1024];
        let src = buffer.as_ref();
        let mut i = 0;
        while i < buffer.len() {
            out[i] = src[i];
            i += 1;
        }
        out
    };
    &ASSET
};

pub fn find_assets(binary: &[u8]) -> Vec<Asset> {
    let prefix = asset_prefix();
    let finder = memmem::Finder::new(&prefix);
    finder
        .find_iter(binary)
        .filter_map(|index| {
            let start = index + prefix.len();
            let buffer = ConstReadBuffer::new(binary.get(start..)?);
            deserialize_const!(Asset, buffer).map(|(_, asset)| asset)
        })
        .collect()
}
