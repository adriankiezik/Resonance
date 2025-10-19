use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PackAssetsConfig {
    pub assets_dir: PathBuf,
    pub output_pak: PathBuf,
    pub compress: bool,
}

impl Default for PackAssetsConfig {
    fn default() -> Self {
        Self {
            assets_dir: PathBuf::from("assets"),
            output_pak: PathBuf::from("game_assets.pak"),
            compress: true,
        }
    }
}

impl PackAssetsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assets_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.assets_dir = dir.into();
        self
    }

    pub fn output_pak(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_pak = path.into();
        self
    }

    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

pub fn pack_assets_auto() {
    pack_assets_with_config(PackAssetsConfig::default());
}

pub fn pack_assets_with_config(config: PackAssetsConfig) {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    if profile != "release" {
        return;
    }

    println!("cargo:warning=Building in RELEASE mode - packing assets...");

    if !config.assets_dir.exists() {
        println!(
            "cargo:warning=Assets directory not found at {:?}, skipping asset packing",
            config.assets_dir
        );
        println!("cargo:warning=Your release build will expect assets in a PAK file!");
        println!(
            "cargo:warning=Create '{}' directory and add your game assets",
            config.assets_dir.display()
        );
        return;
    }

    if is_directory_empty(&config.assets_dir) {
        println!("cargo:warning=Assets directory is empty, skipping asset packing");
        return;
    }

    println!(
        "cargo:warning=Packing assets from {:?} to {:?}",
        config.assets_dir, config.output_pak
    );

    let mut args = vec![
        "run",
        "--package",
        "resonance",
        "--bin",
        "asset-packer",
        "--release",
        "--",
        "--input",
    ];

    let input_str = config.assets_dir.to_string_lossy();
    let output_str = config.output_pak.to_string_lossy();

    args.push(&input_str);
    args.push("--output");
    args.push(&output_str);

    if config.compress {
        args.push("--compress");
    }

    let status = Command::new("cargo").args(&args).status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!(
                    "cargo:warning=✓ Assets packed successfully to {:?}",
                    config.output_pak
                );

                if let Ok(metadata) = std::fs::metadata(&config.output_pak) {
                    let size_mb = metadata.len() as f64 / 1_048_576.0;
                    println!("cargo:warning=  PAK file size: {:.2} MB", size_mb);
                }
            } else {
                println!(
                    "cargo:warning=✗ Asset packing failed with status: {}",
                    exit_status
                );
                println!("cargo:warning=Your release build may not work correctly!");
            }
        }
        Err(e) => {
            println!("cargo:warning=✗ Failed to run asset packer: {}", e);
            println!("cargo:warning=Make sure resonance is available as a dependency");
        }
    }

    println!("cargo:rerun-if-changed={}", config.assets_dir.display());
}

fn is_directory_empty(path: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if entry.is_ok() {
                return false;
            }
        }
    }
    true
}
