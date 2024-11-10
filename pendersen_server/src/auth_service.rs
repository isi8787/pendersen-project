use tonic::{Request, Response, Status};
use num_bigint::{BigInt, Sign};
use rand::Rng;
use serde::Deserialize;
use std::{collections::HashMap, fs, str::FromStr, sync::{Arc, Mutex}};

use crate::pb::{RegisterRequest, RegisterResponse, AuthenticationChallengeRequest, AuthenticationChallengeResponse, AuthenticationAnswerRequest, AuthenticationAnswerResponse};

#[derive(Deserialize)]
struct Params {
    p: String,
    q: String,
    g: String,
    h: String,
}

pub struct AuthService {
    users: Arc<Mutex<HashMap<String, UserPublicParameters>>>,
    sessions: Arc<Mutex<HashMap<String, SessionParameters>>>,
    g: BigInt,
    h: BigInt,
    p: BigInt,
    q: BigInt, // Consider using this field, or remove it if unnecessary
}

#[derive(Clone)]
struct UserPublicParameters {
    y1: BigInt,
    y2: BigInt,
}

#[derive(Clone)]
struct SessionParameters {
    user: String,
    r1: BigInt,
    r2: BigInt,
    c: BigInt,
}

impl AuthService {
    pub fn new(g: BigInt, h: BigInt, p: BigInt, q: BigInt) -> Self {
        AuthService {
            users: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            g,
            h,
            p,
            q,
        }
    }

    pub fn load_parameters() -> Result<(BigInt, BigInt, BigInt, BigInt), Box<dyn std::error::Error>> {
        let file_content = fs::read_to_string("../parameters.json")?;
        let params: Params = serde_json::from_str(&file_content)?;

        let p = BigInt::from_str(&params.p)?;
        let q = BigInt::from_str(&params.q)?;
        let g = BigInt::from_str(&params.g)?;
        let h = BigInt::from_str(&params.h)?;

        Ok((p, q, g, h))
    }
}

#[tonic::async_trait]
impl crate::pb::auth_server::Auth for AuthService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        println!("Received registration for user: {}", req.user);

        let y1 = BigInt::from_str(&req.y1).unwrap_or_default();
        let y2 = BigInt::from_str(&req.y2).unwrap_or_default();

        let user_params = UserPublicParameters { y1, y2 };
        self.users.lock().unwrap().insert(req.user.clone(), user_params);

        let response = RegisterResponse {
            message: format!("User {} registered successfully", req.user),
        };

        Ok(Response::new(response))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        let req = request.into_inner();
        println!("Received authentication challenge for user: {}", req.user);

        let r1 = BigInt::from_str(&req.r1).unwrap_or_default();
        let r2 = BigInt::from_str(&req.r2).unwrap_or_default();

        let c = BigInt::from(rand::thread_rng().gen_range(1..10000)); // Random challenge
        let auth_id = "auth123".to_string();

        let session_params = SessionParameters {
            user: req.user.clone(),
            r1,
            r2,
            c: c.clone(),
        };

        self.sessions.lock().unwrap().insert(auth_id.clone(), session_params);

        let response = AuthenticationChallengeResponse {
            auth_id,
            c: c.to_string(),
        };

        Ok(Response::new(response))
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().unwrap();
        let session = match sessions.get(&req.auth_id) {
            Some(session) => session,
            None => return Err(Status::not_found("Session not found")),
        };

        let users = self.users.lock().unwrap();
        let user_params = match users.get(&session.user) {
            Some(params) => params,
            None => return Err(Status::not_found("User not found")),
        };

        let s = BigInt::from_str(&req.s).unwrap_or_default();

        // Ensure exponents are positive
        if s.sign() == Sign::Minus {
            return Err(Status::invalid_argument("Negative exponentiation is not allowed"));
        }

        // Calculate A' = g^s * y1^c mod p
        let r1p = pow_mod(&self.g, &s, &user_params.y1, &session.c, &self.p);

        // Calculate B' = h^s * y2^c mod p
        let r2p = pow_mod(&self.h, &s, &user_params.y2, &session.c, &self.p);

        // Retrieve R1 and R2 for comparison
        let r1 = &session.r1;
        let r2 = &session.r2;

        let session_id = if r1p == *r1 && r2p == *r2 {
            println!("Verification successful for user: {}", session.user);
            "session123".to_string() // Indicate success
        } else {
            println!("Verification failed for user: {}", session.user);
            "".to_string() // Indicate failure
        };

        let response = AuthenticationAnswerResponse { session_id };

        Ok(Response::new(response))
    }
}

// Modular exponentiation function
fn pow_mod(g: &BigInt, s: &BigInt, y: &BigInt, c: &BigInt, p: &BigInt) -> BigInt {
    let base1 = g.modpow(s, p);
    let base2 = y.modpow(c, p);
    let result = (base1 * base2) % p;
    if result.sign() == Sign::Minus {
        result + p
    } else {
        result
    }
}
