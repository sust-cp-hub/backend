# SUST CP Backend

REST API backend for the SUST Competitive Programming Community Platform.

## Tech Stack

- **Language**: Rust
- **Framework**: Axum
- **Database**: PostgreSQL (Neon)
- **Auth**: JWT + Argon2 password hashing

## Setup

1. Clone the repo
2. Copy `.env.example` to `.env` and fill in your database URL and JWT secret
3. Run the server:

```bash
cargo run
```

The server starts at `http://localhost:8080`.

## API Endpoints

### Health Check
```
GET /api/health
```
