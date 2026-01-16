use anyhow::Result;
use clap::{Parser, Subcommand};
use semver::Version;
use std::fs;

/// AstraWeave Release Automation Tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump version in Cargo.toml files
    Bump {
        /// Version component to bump (major, minor, patch)
        component: String,
    },
    /// Create a release tag
    Tag {
        /// Version to tag
        version: String,
    },
    /// Build and package release artifacts
    Package {
        /// Version to package
        version: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Bump { component } => bump_version(&component),
        Commands::Tag { version } => create_tag(&version),
        Commands::Package { version } => package_release(&version),
    }
}

fn bump_version(component: &str) -> Result<()> {
    // Read current version from root Cargo.toml
    let root_cargo = fs::read_to_string("Cargo.toml")?;
    let mut cargo: toml::Value = toml::from_str(&root_cargo)?;

    if let Some(package) = cargo
        .get_mut("workspace")
        .and_then(|w| w.get_mut("package"))
    {
        if let Some(version) = package.get_mut("version") {
            if let Some(version_str) = version.as_str() {
                let mut ver: Version = version_str.parse()?;
                match component {
                    "major" => {
                        ver.major += 1;
                        ver.minor = 0;
                        ver.patch = 0;
                    }
                    "minor" => {
                        ver.minor += 1;
                        ver.patch = 0;
                    }
                    "patch" => ver.patch += 1,
                    _ => anyhow::bail!("Invalid component: {}", component),
                }
                *version = toml::Value::String(ver.to_string());
            }
        }
    }

    // Write back
    fs::write("Cargo.toml", toml::to_string_pretty(&cargo)?)?;

    println!(
        "Bumped version to {}",
        cargo["workspace"]["package"]["version"]
    );

    Ok(())
}

fn create_tag(version: &str) -> Result<()> {
    // Validate version
    let _ver: Version = version.parse()?;

    // Create annotated tag
    std::process::Command::new("git")
        .args([
            "tag",
            "-a",
            &format!("v{}", version),
            "-m",
            &format!("Release v{}", version),
        ])
        .status()?;

    println!("Created tag v{}", version);

    Ok(())
}

fn package_release(version: &str) -> Result<()> {
    // Validate version
    let _ver: Version = version.parse()?;

    // Build release
    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .status()?;

    // Create package directory
    let package_dir = format!("astraweave-{}", version);
    fs::create_dir_all(&package_dir)?;

    // Copy binaries and assets
    // This is a simplified version - in practice, would copy specific artifacts

    println!("Packaged release v{} in {}", version, package_dir);

    Ok(())
}
