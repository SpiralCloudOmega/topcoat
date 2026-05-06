use std::process::Stdio;

use clap::{Args, Subcommand};
use console::style;
use tokio::process::Command;

#[derive(Args)]
pub struct AssetCommand {
    #[command(subcommand)]
    command: AssetSubcommand,
}

#[derive(Subcommand)]
enum AssetSubcommand {
    /// List all asset paths embedded in the binary produced by cargo
    List(ListArgs),
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

impl AssetCommand {
    pub async fn run(self) {
        match self.command {
            AssetSubcommand::List(args) => list(args).await,
        }
    }
}

async fn list(args: ListArgs) {
    let executable = match build_executable(&args).await {
        Some(path) => path,
        None => std::process::exit(1),
    };

    let bytes = match std::fs::read(&executable) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!(
                "{}",
                style(format!("failed to read {executable}: {error}")).red()
            );
            std::process::exit(1);
        }
    };

    for asset in topcoat_asset::Asset::find_in_binary(&bytes) {
        println!("{:?}", asset.resolved_path());
    }
}

async fn build_executable(args: &ListArgs) -> Option<String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--message-format=json"]);
    if let Some(bin) = &args.bin {
        cmd.args(["--bin", bin]);
    }
    if let Some(package) = &args.package {
        cmd.args(["--package", package]);
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn cargo build")
        .wait_with_output()
        .await
        .expect("failed to wait for cargo build");

    if !output.status.success() {
        eprintln!("{}", style("build failed").red().bold());
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let executables: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let msg: serde_json::Value = serde_json::from_str(line).ok()?;
            if msg.get("reason")?.as_str()? == "compiler-artifact" {
                msg.get("executable")?.as_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    match executables.len() {
        0 => {
            eprintln!("{}", style("no executable produced by cargo build").red());
            None
        }
        1 => Some(executables.into_iter().next().unwrap()),
        _ => {
            eprintln!(
                "{}",
                style("cargo produced multiple binaries; pass --bin or --package to choose one:")
                    .red()
            );
            for exe in &executables {
                eprintln!("  {exe}");
            }
            None
        }
    }
}
