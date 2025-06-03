use std::fs;

fn main() {
    // Iterate over the files in the `proto` directory
    let proto_dir = "proto";
    let paths = fs::read_dir(proto_dir).expect("Failed to read proto directory");
    for path in paths {
        let path = path.expect("Failed to read path").path();
        if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            // Compile the protobuf file
            tonic_build::compile_protos(&path).expect("Failed to compile protobuf file");
        }
    }
}
