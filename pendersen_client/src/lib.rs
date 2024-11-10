use tonic::transport::Channel;
use tonic::{Request, Response};
use num_bigint::BigInt;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::str::FromStr;

// Include the generated gRPC module
pub mod pb {
    tonic::include_proto!("zkp_auth");
}

use pb::{
    auth_client::AuthClient, RegisterRequest, AuthenticationChallengeRequest,
    AuthenticationAnswerRequest, RegisterResponse, AuthenticationChallengeResponse,
    AuthenticationAnswerResponse,
};

// Struct for loading parameters from a JSON file
#[derive(Deserialize)]
pub struct Parameters {
    pub p: String,
    pub q: String,
    pub g: String,
    pub h: String,
}

// Function to load parameters from a JSON file
pub fn load_parameters() -> (BigInt, BigInt, BigInt, BigInt) {
    let file_content = fs::read_to_string("../parameters.json").expect("Unable to read file");
    let params: Parameters = serde_json::from_str(&file_content).expect("Error parsing JSON");

    let p = BigInt::from_str(&params.p).expect("Invalid P value");
    let q = BigInt::from_str(&params.q).expect("Invalid Q value");
    let g = BigInt::from_str(&params.g).expect("Invalid G value");
    let h = BigInt::from_str(&params.h).expect("Invalid H value");

    (p, q, g, h)
}

// Function to read user input from the console
pub fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// Function to perform modular exponentiation
pub fn pow(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    base.modpow(exp, modulus)
}

// Trait for AuthClient to allow mocking in tests
pub trait AuthClientTrait {
    fn register(
        &mut self,
        request: Request<RegisterRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<RegisterResponse>, tonic::Status>> + Send + '_>>;

    fn create_authentication_challenge(
        &mut self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationChallengeResponse>, tonic::Status>> + Send + '_>>;

    fn verify_authentication(
        &mut self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationAnswerResponse>, tonic::Status>> + Send + '_>>;
}

// Implement the trait for the actual AuthClient
impl AuthClientTrait for AuthClient<Channel> {
    fn register(
        &mut self,
        request: Request<RegisterRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<RegisterResponse>, tonic::Status>> + Send + '_>> {
        Box::pin(self.register(request))
    }

    fn create_authentication_challenge(
        &mut self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationChallengeResponse>, tonic::Status>> + Send + '_>> {
        Box::pin(self.create_authentication_challenge(request))
    }

    fn verify_authentication(
        &mut self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<AuthenticationAnswerResponse>, tonic::Status>> + Send + '_>> {
        Box::pin(self.verify_authentication(request))
    }
}

// Struct for the AuthServiceClient
pub struct AuthServiceClient<T> {
    client: T,
}

impl AuthServiceClient<AuthClient<Channel>> {
    pub async fn connect(dst: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AuthClient::connect(dst).await?;
        Ok(Self { client })
    }
}

impl<T: AuthClientTrait + Send + Sync> AuthServiceClient<T> {
    pub async fn register(
        &mut self,
        user_id: &str,
        y1: &BigInt,
        y2: &BigInt,
    ) -> Result<(), tonic::Status> {
        let register_req = RegisterRequest {
            user: user_id.to_string(),
            y1: y1.to_string(),
            y2: y2.to_string(),
        };

        self.client.register(Request::new(register_req)).await?;
        Ok(())
    }

    pub async fn create_authentication_challenge(
        &mut self,
        user_id: &str,
        r1: &BigInt,
        r2: &BigInt,
    ) -> Result<(String, BigInt), tonic::Status> {
        let auth_req = AuthenticationChallengeRequest {
            user: user_id.to_string(),
            r1: r1.to_string(),
            r2: r2.to_string(),
        };

        let response = self.client.create_authentication_challenge(Request::new(auth_req)).await?;
        let auth_res = response.into_inner();
        let c = BigInt::from_str(&auth_res.c).expect("Invalid C value");

        Ok((auth_res.auth_id, c))
    }

    pub async fn verify_authentication(
        &mut self,
        auth_id: &str,
        s: &BigInt,
    ) -> Result<(), tonic::Status> {
        let auth_ans_req = AuthenticationAnswerRequest {
            auth_id: auth_id.to_string(),
            s: s.to_string(),
        };

        let response = self.client.verify_authentication(Request::new(auth_ans_req)).await?;
        let auth_ans_res = response.into_inner();

        if auth_ans_res.session_id.is_empty() {
            println!("Authentication failed");
        } else {
            println!("Authentication succeeded with session ID: {}", auth_ans_res.session_id);
        }
        Ok(())
    }
}
