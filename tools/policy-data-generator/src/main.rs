//! Generate mock data.

use clap::Parser;
use policy_styx_lib::crypto::pcd_crypto_backend_aes_gcm_randkey_generate;
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
        .map(|entry| std::fs::read(entry.path()).expect("Failed to read file"))
        .collect::<Vec<_>>();

    let key = pcd_crypto_backend_aes_gcm_randkey_generate().expect("Failed to generate random key");
    let data = pcd_dataset_pack_data_multiple(files, &key).expect("Failed to pack data");

    // Write to the output.
    let output_file = format!("{}/data.enc", args.output);
    let data = bincode::serialize(&data).expect("Failed to serialize data");
    std::fs::write(&output_file, &data).expect("Failed to write data to file");

    // Write the key to a separate file.
    let key_file = format!("{}/key", args.output);
    std::fs::write(&key_file, key).expect("Failed to write key to file");
}
