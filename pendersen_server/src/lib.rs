// src/lib.rs

pub mod pb {
    tonic::include_proto!("zkp_auth"); // Adjust "zkp_auth" to match your .proto package name
}

pub mod auth_service;
pub use auth_service::AuthService;
