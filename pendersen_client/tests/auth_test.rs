use pendersen_client::*;
use pendersen_client::pb::{
    RegisterRequest, AuthenticationChallengeRequest, AuthenticationAnswerRequest,
    RegisterResponse, AuthenticationChallengeResponse, AuthenticationAnswerResponse,
};
use tonic::{Request, Response};
use num_bigint::BigInt;
use num_traits::Num; // Import the Num trait for from_str_radix
use mockall::mock;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    // Define the mock AuthClientTrait using mockall
    mock! {
        pub AuthClientTrait {
            fn register(
                &mut self,
                req: Request<RegisterRequest>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<RegisterResponse>, tonic::Status>> + Send>>;

            fn create_authentication_challenge(
                &mut self,
                req: Request<AuthenticationChallengeRequest>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationChallengeResponse>, tonic::Status>> + Send>>;

            fn verify_authentication(
                &mut self,
                req: Request<AuthenticationAnswerRequest>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationAnswerResponse>, tonic::Status>> + Send>>;
        }
    }

    // Implement AuthClientTrait for MockAuthClientTrait
    impl AuthClientTrait for MockAuthClientTrait {
        fn register(
            &mut self,
            req: Request<RegisterRequest>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<RegisterResponse>, tonic::Status>> + Send>> {
            self.register(req)
        }

        fn create_authentication_challenge(
            &mut self,
            req: Request<AuthenticationChallengeRequest>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationChallengeResponse>, tonic::Status>> + Send>> {
            self.create_authentication_challenge(req)
        }

        fn verify_authentication(
            &mut self,
            req: Request<AuthenticationAnswerRequest>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationAnswerResponse>, tonic::Status>> + Send>> {
            self.verify_authentication(req)
        }
    }

    // Modify AuthServiceClient to be generic over the client type
    struct AuthServiceClient<T> {
        client: T,
    }

    impl<T: AuthClientTrait + Send + Sync> AuthServiceClient<T> {
        async fn register(
            &mut self,
            user_id: &str,
            y1: &BigInt,
            y2: &BigInt,
        ) -> Result<(), tonic::Status> {
            let request = Request::new(RegisterRequest {
                user: user_id.to_string(),
                y1: y1.to_string(),
                y2: y2.to_string(),
            });
            self.client.register(request).await?;
            Ok(())
        }

        async fn create_authentication_challenge(
            &mut self,
            user_id: &str,
            r1: &BigInt,
            r2: &BigInt,
        ) -> Result<(String, BigInt), tonic::Status> {
            let request = Request::new(AuthenticationChallengeRequest {
                user: user_id.to_string(),
                r1: r1.to_string(),
                r2: r2.to_string(),
            });
            let response = self.client.create_authentication_challenge(request).await?;
            let reply = response.into_inner();
            let c = BigInt::from_str_radix(&reply.c, 10).expect("Invalid C value");
            Ok((reply.auth_id, c))
        }

        async fn verify_authentication(
            &mut self,
            auth_id: &str,
            s: &BigInt,
        ) -> Result<(), tonic::Status> {
            let request = Request::new(AuthenticationAnswerRequest {
                auth_id: auth_id.to_string(),
                s: s.to_string(),
            });
            self.client.verify_authentication(request).await?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_user() {
        let mut mock_client = MockAuthClientTrait::new();

        mock_client
            .expect_register()
            .withf(|req: &Request<RegisterRequest>| req.get_ref().user == "test_user")
            .returning(|_| Box::pin(async { Ok(Response::new(RegisterResponse { message: "User registered".to_string() })) }));

        let mut auth_service_client = AuthServiceClient { client: mock_client };

        let user_id = "test_user";
        let y1 = BigInt::from(12345);
        let y2 = BigInt::from(67890);

        let result = auth_service_client.register(user_id, &y1, &y2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_authentication_challenge() {
        let mut mock_client = MockAuthClientTrait::new();

        mock_client
            .expect_create_authentication_challenge()
            .withf(|req: &Request<AuthenticationChallengeRequest>| req.get_ref().user == "test_user")
            .returning(|_| Box::pin(async { Ok(Response::new(AuthenticationChallengeResponse { c: "12345".to_string(), auth_id: "auth123".to_string() })) }));

        let mut auth_service_client = AuthServiceClient { client: mock_client };

        let user_id = "test_user";
        let r1 = BigInt::from(12345);
        let r2 = BigInt::from(67890);

        let (auth_id, c) = auth_service_client.create_authentication_challenge(user_id, &r1, &r2).await.unwrap();
        assert_eq!(auth_id, "auth123");
        assert_eq!(c, BigInt::from(12345));
    }

    #[tokio::test]
    async fn test_verify_authentication() {
        let mut mock_client = MockAuthClientTrait::new();

        mock_client
            .expect_verify_authentication()
            .withf(|req: &Request<AuthenticationAnswerRequest>| req.get_ref().auth_id == "auth123")
            .returning(|_| Box::pin(async { Ok(Response::new(AuthenticationAnswerResponse { session_id: "session123".to_string() })) }));

        let mut auth_service_client = AuthServiceClient { client: mock_client };

        let auth_id = "auth123";
        let s = BigInt::from(12345);

        let result = auth_service_client.verify_authentication(auth_id, &s).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_authentication_with_bad_password() {
        let mut mock_client = MockAuthClientTrait::new();

        // Simulate a failure response when a bad password is entered
        mock_client
            .expect_verify_authentication()
            .withf(|req: &Request<AuthenticationAnswerRequest>| req.get_ref().auth_id == "auth123")
            .returning(|_| Box::pin(async { Err(tonic::Status::unauthenticated("Bad password")) }));

        let mut auth_service_client = AuthServiceClient { client: mock_client };

        let auth_id = "auth123";
        let s = BigInt::from(54321); // Bad password simulation

        let result = auth_service_client.verify_authentication(auth_id, &s).await;
        assert!(result.is_err());
        if let Err(status) = result {
            assert_eq!(status.code(), tonic::Code::Unauthenticated);
            assert_eq!(status.message(), "Bad password");
        }
    }
}
