fn main() {
    let proto_dir = "proto";
    let proto_files = [
        "proto/d1doctor/v1/common.proto",
        "proto/d1doctor/v1/messages.proto",
        "proto/d1doctor/v1/agent.proto",
        "proto/d1doctor/v1/auth.proto",
        "proto/d1doctor/v1/credits.proto",
    ];

    prost_build::Config::new()
        .compile_protos(&proto_files, &[proto_dir])
        .expect("Failed to compile proto files");

    println!("cargo:rerun-if-changed=proto/");
}
