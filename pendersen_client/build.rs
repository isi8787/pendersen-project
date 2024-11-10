fn main() {
    tonic_build::configure()
        .build_server(true)  // Generate server code
        .build_client(true)  // Generate client code
        .compile(&["../proto/pendersen.proto"], &["../proto"])
        .expect("Failed to compile proto files");
}
