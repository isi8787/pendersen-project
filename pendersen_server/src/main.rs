use tonic::transport::Server;
use std::error::Error;

mod pb {
    tonic::include_proto!("zkp_auth");
}

mod auth_service;
use auth_service::AuthService;
use pb::auth_server::AuthServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (p, q, g, h) = AuthService::load_parameters()?;
    let addr = "[::1]:50051".parse()?;
    let auth_service = AuthService::new(g, h, p, q);

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
