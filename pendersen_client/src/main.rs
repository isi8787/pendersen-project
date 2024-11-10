// src/main.rs

use pendersen_client::{load_parameters, read_input, pow, AuthServiceClient};
use num_bigint::BigInt;
use num_integer::Integer;
use rand::Rng;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (p, q, g, h) = load_parameters();

    let mut client = AuthServiceClient::connect("http://[::1]:50051".to_string()).await?;

    // Register User
    let user_id = read_input("Enter UserID: ");
    let password_str = read_input("Enter Password: ");
    let password = BigInt::from_str(&password_str).expect("Invalid password");

    let y1 = pow(&g, &password, &p);
    let y2 = pow(&h, &password, &p);

    client.register(&user_id, &y1, &y2).await?;

    // Authentication Challenge
    let login = read_input("Do you want to login? (yes/no): ");
    if login.to_lowercase() == "yes" {
        let ran_k: i64 = rand::thread_rng().gen_range(1..10000);
        let k = BigInt::from(ran_k);

        let r1 = pow(&g, &k, &p);
        let r2 = pow(&h, &k, &p);

        let (auth_id, c) = client.create_authentication_challenge(&user_id, &r1, &r2).await?;

        // Prompt the user to reenter the password for authentication
        let reentered_password_str = read_input("Reenter Password: ");
        let reentered_password = BigInt::from_str(&reentered_password_str).expect("Invalid password");

        // Calculate s = (k - c * reentered_password) % q, ensuring it's non-negative
        let s_2 = &reentered_password * &c;
        let s = (k - s_2).mod_floor(&q);

        println!("Calculated s value: {}", s);

        client.verify_authentication(&auth_id, &s).await?;
    } else {
        println!("Exiting program.");
    }

    Ok(())
}
