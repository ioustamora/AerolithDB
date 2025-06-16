fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../proto/aerolithdb.proto";
    let proto_dir = "../proto";

    // Tell Cargo to recompile if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);

    // Try to compile Protocol Buffers, but don't fail if protoc is not available
    match tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile(&[proto_file], &[proto_dir])
    {
        Ok(_) => {
            println!("cargo:warning=Successfully compiled Protocol Buffers");
        }
        Err(e) => {
            println!("cargo:warning=Protocol Buffers compilation failed: {}. Install protoc to enable full gRPC support.", e);
            println!("cargo:warning=gRPC v2 API will use manual types instead of generated Protocol Buffer types.");
            
            // Create empty proto module to prevent compilation errors
            std::fs::create_dir_all("src/proto")?;
            std::fs::write("src/proto/mod.rs", 
                "// Protocol Buffers not available - using manual types\n\
                 // Install protoc to enable generated Protocol Buffer types\n")?;
        }
    }

    Ok(())
}
