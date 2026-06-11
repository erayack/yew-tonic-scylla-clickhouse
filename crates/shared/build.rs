fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_server = std::env::var_os("CARGO_FEATURE_SERVER").is_some();

    tonic_build::configure()
        .build_server(build_server)
        .build_client(false)
        .compile_protos(&["../../proto/app.proto"], &["../../proto"])?;
    println!("cargo:rerun-if-changed=../../proto/app.proto");
    Ok(())
}
