# Secure Tor Marketplace

A secure, privacy-focused marketplace built on the Tor network.

## Features

- Anonymous registration (no email required)
- PGP-based 2FA
- Multi-signature escrow system
- End-to-end encrypted messaging
- Cryptocurrency support (BTC, XMR)
- Tor network integration

## Running the Application

### Prerequisites

- Rust and Cargo (https://www.rust-lang.org/tools/install)

### Quick Start

1. Run the application using the provided script:

```bash
./run_app.sh
```

2. Open your browser and navigate to `http://localhost:3000`

3. You can also view specific pages:
   - Home: http://localhost:3000/
   - Login: http://localhost:3000/login
   - Register: http://localhost:3000/register
   - Products: http://localhost:3000/products

### Manual Start

If the script doesn't work, you can run the application manually:

```bash
# Build the application
cargo build

# Run the application
cargo run
```

## Project Structure

- `src/` - Rust source code
- `static/` - Static HTML, CSS, and image files
- `templates/` - HTML templates (for future implementation)

## Current Status

This is a simplified version of the application with static HTML pages. The full version will include:

- Database integration with PostgreSQL
- User authentication with JWT and PGP-based 2FA
- Product management
- Order processing
- Encrypted messaging system
- Cryptocurrency wallet integration
- Tor network integration

### Project structure

```bash
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config
│   ├── default.json    # Default configuration
│   ├── production.json # Production configuration (Overwrites the default)
│   └── test.json       # Test configuration (Overwrites the default)
├── rustfmt.toml
├── src
│   ├── database.rs
│   ├── errors.rs
│   ├── lib             # Helpers not related to the business model
│   │   ├── authenticate_request.rs
│   │   ├── date.rs
│   │   ├── mod.rs
│   │   ├── models.rs   # Base Database Model trait
│   │   ├── to_object_id.rs
│   │   └── token.rs
│   ├── logger.rs
│   ├── main.rs
│   ├── models
│   │   ├── cat.rs
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── routes
│   │   ├── cat.rs
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   └── user.rs
│   ├── settings.rs
│   └── tests           # E2E Tests
└── test.sh
```

### Test
To run tests make sure MongoDB is up and running.
```
make test
```

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.
