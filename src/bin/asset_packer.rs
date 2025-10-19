use resonance::assets::{PakBuilder, PakError};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let mut input_path = PathBuf::new();
    let mut output_path = PathBuf::new();
    let mut compress = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" | "-i" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --input requires a path");
                    std::process::exit(1);
                }
                input_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--output" | "-o" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --output requires a path");
                    std::process::exit(1);
                }
                output_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--compress" | "-c" => {
                compress = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    if input_path.as_os_str().is_empty() {
        eprintln!("Error: --input is required");
        print_usage();
        std::process::exit(1);
    }

    if output_path.as_os_str().is_empty() {
        eprintln!("Error: --output is required");
        print_usage();
        std::process::exit(1);
    }

    println!("Resonance Asset Packer");
    println!("======================");
    println!("Input:   {}", input_path.display());
    println!("Output:  {}", output_path.display());
    println!("Compress: {}", if compress { "yes" } else { "no" });
    println!();

    let start = Instant::now();

    pack_directory(&input_path, &output_path, compress)?;

    let elapsed = start.elapsed();
    println!("\nPacking completed in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

fn print_usage() {
    println!("Resonance Asset Packer");
    println!("Packs game assets into a .pak archive file");
    println!();
    println!("USAGE:");
    println!("    asset-packer --input <DIR> --output <FILE> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -i, --input <DIR>     Input directory containing assets");
    println!("    -o, --output <FILE>   Output PAK file path");
    println!("    -c, --compress        Enable compression (deflate)");
    println!("    -h, --help            Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    asset-packer -i ./assets -o game_assets.pak");
    println!("    asset-packer -i ./assets -o game_assets.pak --compress");
}

fn pack_directory(input_path: &Path, output_path: &Path, compress: bool) -> Result<(), PakError> {
    if !input_path.exists() {
        eprintln!(
            "Error: Input directory does not exist: {}",
            input_path.display()
        );
        std::process::exit(1);
    }

    if !input_path.is_dir() {
        eprintln!(
            "Error: Input path is not a directory: {}",
            input_path.display()
        );
        std::process::exit(1);
    }

    let mut builder = PakBuilder::new().with_compression(compress);

    println!("Scanning directory...");

    let files = collect_files(input_path)?;

    if files.is_empty() {
        println!("Warning: No files found in input directory");
        return Ok(());
    }

    println!("Found {} files to pack\n", files.len());

    let mut total_size = 0u64;

    for (file_path, relative_path) in &files {
        let metadata = std::fs::metadata(file_path)?;
        let size = metadata.len();
        total_size += size;

        println!("  {} ({} bytes)", relative_path, size);

        builder.add_file(relative_path.clone(), file_path)?;
    }

    println!(
        "\nTotal size: {} bytes ({:.2} MB)",
        total_size,
        total_size as f64 / 1_048_576.0
    );
    println!("\nBuilding PAK archive...");

    builder.build(output_path)?;

    let output_metadata = std::fs::metadata(output_path)?;
    let output_size = output_metadata.len();
    let compression_ratio = if compress && total_size > 0 {
        100.0 - (output_size as f64 / total_size as f64 * 100.0)
    } else {
        0.0
    };

    println!(
        "Output size: {} bytes ({:.2} MB)",
        output_size,
        output_size as f64 / 1_048_576.0
    );

    if compress {
        println!("Compression: {:.1}%", compression_ratio);
    }

    Ok(())
}

fn collect_files(dir: &Path) -> Result<Vec<(PathBuf, String)>, PakError> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<(PathBuf, String)>,
) -> Result<(), PakError> {
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(base_dir, &path, files)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(base_dir).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to strip prefix: {}", e),
                )
            })?;

            let relative_str = relative.to_string_lossy().replace('\\', "/");

            files.push((path, relative_str));
        }
    }

    Ok(())
}
