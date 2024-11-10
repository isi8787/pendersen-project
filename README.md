# Penderson Project

## Overview
The **Penderson Project** simulates a secure login mechanism using a zero-knowledge proof approach. It is implemented in Rust and uses **gRPC** for client-server communication based on a shared `.proto` file.

### Project Structure
- **pendersen_client**: The client-side application for registering and authenticating a user.
- **pendersen_server**: The server-side application that handles user registration and authentication requests.
- **proto**: Directory containing the `.proto` file used for defining the gRPC communication structure.

## Features
- **Zero-Knowledge Proof**: Password-based authentication using a cryptographic zero-knowledge approach.
- **gRPC Communication**: Interactions between the client and server are facilitated using gRPC.
- **Tests**: Each project contains unit tests that can be executed using `cargo test`.

## Getting Started

### Prerequisites
- **Rust**: Ensure you have Rust installed. You can install it from [rust-lang.org](https://www.rust-lang.org).
- **Protobuf Compiler**: Make sure `protoc` is installed for generating Rust gRPC code.

### Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/pendersen-project.git
   cd pendersen-project
