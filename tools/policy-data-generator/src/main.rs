//! Generate mock data.

use clap::Parser;
use policy_styx_lib::dataset::pcd_dataset_pack_data_multiple;

#[derive(Parser, Clone, Debug)]
struct Args {
    /// The path to the output directory.
    #[clap(short, long, default_value = "./data")]
    output: String,

    /// The path to the input file.
    #[clap(short, long, default_value = "./data")]
    input: String,

    /// The input file extension.
    #[clap(short, long, default_value = "arrow")]
    input_ext: String,
}

fn main() {
    let args = Args::parse();

    // Check if the output path is a directory.
    if !std::fs::metadata(&args.output)
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        eprintln!("Output path must be a directory: {}", args.output);
        return;
    }

    // Create the output directory if it doesn't exist.
    std::fs::create_dir_all(&args.output).expect("Failed to create output directory");

    let files = std::fs::read_dir(&args.input)
        .expect("Failed to read input directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext.to_str().unwrap() == args.input_ext)
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        eprintln!("No files found with the extension: {}", args.input_ext);
        return;
    }

    // Process each file.
    let files = files
        .into_iter()
        .map(|entry| {
            // Extract the file name without the extension.
            let name = entry
                .file_name()
                .to_str()
                .expect("Failed to convert file name to string")
                .replace(&format!(".{}", args.input_ext), "");
            let data = std::fs::read(entry.path()).expect("Failed to read file");
            (name, data)
        })
        .collect::<Vec<_>>();

    let data = pcd_dataset_pack_data_multiple(files).expect("Failed to pack data");

    // Write to the output.
    let packed_path = format!("{}/data.bin", args.output);
    std::fs::write(&packed_path, &data).expect("Failed to write packed data");
    println!("Packed data written to: {}", packed_path);
}
