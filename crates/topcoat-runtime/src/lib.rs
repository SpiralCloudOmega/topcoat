mod expr;
mod signal;

pub use expr::*;
pub use signal::*;

use topcoat_asset::{Asset, asset};

pub const SCRIPT: Asset = asset!("browser/dist/index.mjs", rename: "topcoat");
