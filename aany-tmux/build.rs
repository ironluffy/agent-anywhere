fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only compile proto if the feature is enabled
    if std::env::var("CARGO_FEATURE_HUB_CONNECTOR").is_ok() {
        tonic_build::configure()
            .build_server(false) // We only need client code
            .compile(
                &["../aany-hub/src/aany_hub/proto/agent.proto"],
                &["../aany-hub/src/aany_hub/proto"],
            )?;
    }
    Ok(())
}