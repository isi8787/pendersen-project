use pendersen_server::{AuthService, pb};
use pendersen_server::pb::{
    RegisterRequest, AuthenticationChallengeRequest, AuthenticationAnswerRequest,
};
use pendersen_server::pb::auth_server::Auth; // Import the Auth trait

use tokio;
use tonic::Request;
use num_bigint::{BigInt};
use num_traits::One;



#[tokio::test]
async fn test_register_user() {
    // Arrange
    let (G, H, P, Q) = (
        BigInt::from(2),
        BigInt::from(3),
        BigInt::from(17),
        BigInt::from(19),
    );
    let auth_service = AuthService::new(G, H, P, Q);
    let request = Request::new(RegisterRequest {
        user: "test_user".to_string(),
        y1: "123".to_string(),
        y2: "456".to_string(),
    });

    // Act
    let response = auth_service.register(request).await.unwrap();
    let message = response.into_inner().message;

    // Assert
    assert_eq!(message, "User test_user registered successfully");
}

#[tokio::test]
async fn test_create_authentication_challenge() {
    // Arrange
    let (G, H, P, Q) = (
        BigInt::from(2),
        BigInt::from(3),
        BigInt::from(17),
        BigInt::from(19),
    );
    let auth_service = AuthService::new(G, H, P, Q);
    let register_request = Request::new(RegisterRequest {
        user: "test_user".to_string(),
        y1: "123".to_string(),
        y2: "456".to_string(),
    });
    auth_service.register(register_request).await.unwrap();

    let auth_request = Request::new(AuthenticationChallengeRequest {
        user: "test_user".to_string(),
        r1: "789".to_string(),
        r2: "101112".to_string(),
    });

    // Act
    let response = auth_service
        .create_authentication_challenge(auth_request)
        .await
        .unwrap();
    let auth_id = response.into_inner().auth_id;

    // Assert
    assert_eq!(auth_id, "auth123");
}