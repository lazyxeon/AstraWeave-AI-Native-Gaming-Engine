use anyhow::Result;
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
    fs::create_dir_all(output)?;

    // Copy built binaries
    let target_dir = Path::new("target/release");
    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_none() {
            // Assume binary
            let dest = Path::new(output).join(path.file_name().unwrap());
            fs::copy(&path, &dest)?;
        }
    }

    // Copy assets
    copy_dir("assets", &format!("{}/assets", output))?;

    println!("Packaged demos to: {}", output);
    Ok(())
}

fn copy_dir(src: &str, dst: &str) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(src_path.strip_prefix(src)?);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}
