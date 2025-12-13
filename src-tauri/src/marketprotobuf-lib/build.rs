fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = ["proto/market_trading.proto"];

    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={proto_file}");
    }

    // Pure-Rust proto compilation (no external `protoc` needed).
    let file_descriptor_set = protox::Compiler::new(["proto"])?
        .include_source_info(true)
        .include_imports(true)
        .open_files(proto_files)?
        .file_descriptor_set();

    prost_build::Config::new().compile_fds(file_descriptor_set)?;

    Ok(())
}
