fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate the gRPC server (and message types) from the shared contract.
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/poker/v1/poker.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
