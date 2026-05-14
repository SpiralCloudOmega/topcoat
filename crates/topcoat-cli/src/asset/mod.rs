use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;

use crate::cargo::BuildOpts;

const OUT_SUBDIR: &str = "assets";
const CACHE_SUBDIR: &str = "topcoat/cache/assets";

#[derive(Args)]
pub struct AssetCommand {
    #[command(subcommand)]
    command: AssetSubcommand,
}

#[derive(Subcommand)]
enum AssetSubcommand {
    /// List all asset paths embedded in the binary produced by cargo
    List(ListArgs),
    /// Bundle all assets embedded in the binary into a directory
    Bundle(BundleArgs),
    /// Delete the asset bundle directory and the asset build cache
    Clean(CleanArgs),
}

#[derive(Args)]
struct ListArgs {
    /// Build and inspect the named binary target
    #[arg(long)]
    bin: Option<String>,
    /// Build and inspect the named package
    #[arg(short, long)]
    package: Option<String>,
}

#[derive(Args)]
struct BundleArgs {
    /// Build and inspect the named binary target
    #[arg(long)]
    bin: Option<String>,
    /// Build and inspect the named package
    #[arg(short, long)]
    package: Option<String>,
    /// Output directory for the bundle (defaults to <cargo-target>/assets)
    #[arg(short, long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct CleanArgs {
    /// Asset bundle directory to remove (defaults to <cargo-target>/assets)
    #[arg(short, long)]
    out: Option<PathBuf>,
}

impl AssetCommand {
    pub async fn run(self) {
        match self.command {
            AssetSubcommand::List(args) => list(args).await,
            AssetSubcommand::Bundle(args) => bundle(args).await,
            AssetSubcommand::Clean(args) => clean(args).await,
        }
    }
}

async fn list(args: ListArgs) {
    let opts = BuildOpts {
        bin: args.bin,
        package: args.package,
    };
    let (_, bytes) = crate::cargo::build_and_read(&opts)
        .await
        .unwrap_or_else(|e| e.print_and_exit());

    for asset in topcoat_asset::RawAsset::find_in_binary(&bytes) {
        match asset.source() {
            topcoat_asset::Source::Path(p) => {
                println!("{}", p.to_str().unwrap_or("<non-utf8 file path>"))
            }
            topcoat_asset::Source::Url(uri) => println!("{uri}"),
        }
    }
}

async fn bundle(args: BundleArgs) {
    let opts = BuildOpts {
        bin: args.bin,
        package: args.package,
    };
    let (_, bytes) = crate::cargo::build_and_read(&opts)
        .await
        .unwrap_or_else(|e| e.print_and_exit());

    let out_dir = match run_bundle(&bytes, args.out).await {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "{}",
                style(format!("failed to bundle assets: {error}")).red()
            );
            std::process::exit(1);
        }
    };

    println!("bundled assets into {}", out_dir.display());
}

pub(crate) async fn run_bundle(
    bytes: &[u8],
    out_override: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let target_dir = crate::cargo::target_dir()
        .await
        .ok_or("could not derive cargo target directory")?;
    let out_dir = out_override.unwrap_or_else(|| target_dir.join(OUT_SUBDIR));
    let cache_dir = target_dir.join(CACHE_SUBDIR);
    topcoat_asset::Bundler::new(cache_dir)
        .bundle(bytes, &out_dir)
        .await?;
    Ok(out_dir)
}

async fn clean(args: CleanArgs) {
    let target_dir = match crate::cargo::target_dir().await {
        Some(path) => path,
        None => {
            eprintln!(
                "{}",
                style("could not derive cargo target directory; pass --out").red()
            );
            std::process::exit(1);
        }
    };

    let out_dir = args.out.unwrap_or_else(|| target_dir.join(OUT_SUBDIR));
    let cache_dir = target_dir.join(CACHE_SUBDIR);

    for dir in [&out_dir, &cache_dir] {
        match std::fs::remove_dir_all(dir) {
            Ok(()) => println!("removed {}", dir.display()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                eprintln!(
                    "{}",
                    style(format!("failed to remove {}: {error}", dir.display())).red()
                );
                std::process::exit(1);
            }
        }
    }
}
