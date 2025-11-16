use anyhow::Result;
use astraweave_security::path::safe_under;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// AstraWeave Demo Builder Tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all demo examples
    BuildAll,
    /// Build specific demo
    Build {
        /// Demo name
        name: String,
    },
    /// Package demos for distribution
    Package {
        /// Output directory
        output: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::BuildAll => build_all_demos(),
        Commands::Build { name } => build_demo(&name),
        Commands::Package { output } => package_demos(&output),
    }
}

fn build_all_demos() -> Result<()> {
    let examples_dir = Path::new("examples");

    for entry in WalkDir::new(examples_dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            let demo_name = entry.file_name().to_string_lossy();
            println!("Building demo: {}", demo_name);
            build_demo(&demo_name)?;
        }
    }

    println!("All demos built successfully");
    Ok(())
}

fn build_demo(name: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(&["build", "--release", "--bin", name])
        .current_dir("examples")
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to build demo: {}", name);
    }

    println!("Built demo: {}", name);
    Ok(())
}

fn package_demos(output: &str) -> Result<()> {
    // Security: Validate output path is within current directory
    let base = std::env::current_dir()?;
    let output_path = Path::new(output);

    let safe_output = safe_under(&base, output_path)
        .map_err(|e| anyhow::anyhow!("Invalid output path: {}", e))?;

    fs::create_dir_all(&safe_output)?;

    // Copy built binaries
    let target_dir = Path::new("target/release");
    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_none() {
            // Assume binary
            let dest = safe_output.join(path.file_name().unwrap());
            fs::copy(&path, &dest)?;
        }
    }

    // Copy assets
    let assets_dest = safe_output.join("assets");
    copy_dir("assets", &assets_dest)?;

    println!("Packaged demos to: {}", safe_output.display());
    Ok(())
}

fn copy_dir(src: &str, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let relative = src_path.strip_prefix(src)?;
        let dst_path = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}
