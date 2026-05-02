<div align="center">

# SUST CP Geeks Backend

**High-performance REST API powering the SUST Competitive Programming Community Platform**

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue?style=for-the-badge)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)](https://neon.tech/)
[![JWT](https://img.shields.io/badge/JWT-000000?style=for-the-badge&logo=jsonwebtokens&logoColor=white)](https://jwt.io/)

</div>

---

## Architecture

Built on an **enterprise-grade multi-layer architecture** for clean separation of concerns:

```
src/
├── main.rs                 # entry point, router assembly, cors
├── app_state.rs            # shared application state (db pool)
├── errors.rs               # unified AppError enum + IntoResponse
├── validation.rs           # input validation helpers
├── config/
│   └── database.rs         # neon postgres connection pool
├── models/
│   ├── user.rs             # User, RegisterInput, LoginInput, UpdateProfile
│   ├── contest.rs          # Contest, CreateContest, UpdateContest
│   ├── announcement.rs     # Announcement, CreateAnnouncement, UpdateAnnouncement
│   └── event.rs            # Event, Team, TeamMember, response types
├── handlers/
│   ├── auth_handler.rs     # register, login, verify-otp, resend-otp
│   ├── user_handler.rs     # get_me, update_me
│   ├── admin_handler.rs    # user approval, rejection, banning
│   ├── contest_handler.rs  # contest CRUD
│   ├── announcement_handler.rs  # announcement CRUD
│   ├── event_handler.rs    # event + team CRUD
│   └── health_handler.rs   # health check
├── services/
│   └── email.rs            # OTP email sending via Resend API
├── middleware/
│   └── auth_middleware.rs  # JWT claims extractor (FromRequestParts)
├── routes/                 # route definitions per resource
└── utils/
    ├── jwt.rs              # token creation + verification
    └── otp.rs              # OTP generation, storage, verification
```

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Runtime** | Tokio | Async runtime with work-stealing scheduler |
| **Framework** | Axum 0.8 | Ergonomic, type-safe HTTP framework |
| **Database** | PostgreSQL (Neon) | Serverless Postgres with connection pooling |
| **ORM** | SQLx | Compile-time checked SQL queries |
| **Auth** | JWT + Argon2id | Stateless authentication with memory-hard hashing |
| **Email** | Resend | Transactional email for OTP verification |
| **Logging** | tracing | Structured, async-aware request logging |
| **CORS** | tower-http | Cross-origin resource sharing middleware |

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [PostgreSQL](https://neon.tech/) database (Neon recommended)

### Setup

```bash
# clone the repo
git clone git@github.com:sust-cp-geeks/cp-geeks-backend.git
cd cp-geeks-backend

# configure environment
cp .env.example .env
# edit .env with your DATABASE_URL and JWT_SECRET

# run the server
cargo run
```

The server starts at **`http://localhost:8080`**

## API Reference

### Authentication

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `POST` | `/api/auth/register` | Public | Create account + send OTP email |
| `POST` | `/api/auth/verify-otp` | Public | Verify email with 6-digit code |
| `POST` | `/api/auth/resend-otp` | Public | Resend verification code |
| `POST` | `/api/auth/login` | Public | Login & receive JWT token |

### User Profile

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/users/me` | User | View own profile |
| `PUT` | `/api/users/me` | User | Update name, VJudge/CF handles |

### Contests

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/contests` | User | List all contests |
| `GET` | `/api/contests/{id}` | User | View a contest |
| `POST` | `/api/contests` | Admin | Create a contest |
| `PUT` | `/api/contests/{id}` | Admin | Update a contest |
| `DELETE` | `/api/contests/{id}` | Admin | Delete a contest |

### Announcements

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/announcements` | User | List all announcements |
| `GET` | `/api/announcements/{id}` | User | View an announcement |
| `POST` | `/api/announcements` | Admin | Create an announcement |
| `PUT` | `/api/announcements/{id}` | Admin | Update an announcement |
| `DELETE` | `/api/announcements/{id}` | Admin | Delete an announcement |

### Admin Panel

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/admin/users` | Admin | List users (filter: `?status=pending`) |
| `GET` | `/api/admin/users/{id}` | Admin | View user details |
| `PUT` | `/api/admin/users/{id}/approve` | Admin | Approve a pending user |
| `PUT` | `/api/admin/users/{id}/reject` | Admin | Reject a pending user |
| `PUT` | `/api/admin/users/{id}/ban` | Admin | Ban an active user |

### Events

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/events` | User | List all events with teams |
| `GET` | `/api/events/{id}` | User | View a single event |
| `POST` | `/api/events` | Admin/Manager | Create an event |
| `PUT` | `/api/events/{id}` | Admin/Manager | Update an event |
| `DELETE` | `/api/events/{id}` | Admin/Manager | Delete an event |
| `POST` | `/api/events/{id}/teams` | Admin/Manager | Add a team (3 members) |
| `PUT` | `/api/events/{id}/teams/{tid}` | Admin/Manager | Update a team |
| `DELETE` | `/api/events/{id}/teams/{tid}` | Admin/Manager | Delete a team |

### System

| Method | Endpoint | Access | Description |
|--------|----------|--------|-------------|
| `GET` | `/api/health` | Public | Server + database health check |

> **30 endpoints** total — Public (5), User-authenticated (10), Admin/Manager (15)

## Security

- **Argon2id** password hashing (memory-hard, salt-per-user, 19 MB RAM cost)
- **Email OTP verification** — 6-digit code with 10-minute expiry, single-use
- **JWT** stateless authentication with 7-day expiry
- **HMAC-SHA256** token signing
- **Parameterized SQL** — zero SQL injection surface
- **User enumeration prevention** — same error for wrong email vs wrong password
- **Self-ban protection** — admins cannot lock themselves out
- **Input validation** — length limits, email format, URL format checked before DB queries
- **`#[serde(skip_serializing)]`** — password hashes never appear in API responses

## Request Flow

```
Client Request
     │
     ▼
┌─────────────┐
│  CORS Layer │──── Validates origin
└──────┬──────┘
       ▼
┌─────────────┐
│   Router    │──── Matches path → handler
└──────┬──────┘
       ▼
┌─────────────┐
│  Claims     │──── Extracts & verifies JWT from Authorization header
│  Extractor  │     (skipped for public routes)
└──────┬──────┘
       ▼
┌─────────────┐
│  Handler    │──── Validates input → queries DB → returns JSON
└──────┬──────┘
       ▼
  JSON Response
```

## Testing

```bash
# 1. register (sends OTP to your email)
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"reg_number": "2021331001", "name": "Neel", "email": "neel@student.sust.edu", "password": "test123456"}'

# 2. verify your email with the 6-digit code
curl -X POST http://localhost:8080/api/auth/verify-otp \
  -H "Content-Type: application/json" \
  -d '{"email": "neel@student.sust.edu", "code": "123456"}'

# 3. login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "neel@student.sust.edu", "password": "test123456"}'

# 4. use the returned token for protected routes
curl http://localhost:8080/api/users/me \
  -H "Authorization: Bearer <TOKEN>"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | Neon PostgreSQL connection string |
| `JWT_SECRET` | Secret key for signing JWT tokens |
| `RESEND_API_KEY` | Resend API key for sending emails |
| `RESEND_FROM_EMAIL` | Sender address (e.g. `SUST CP Geeks <noreply@yourdomain.com>`) |

## License

MIT

---

<div align="center">

**Built with Rust by [SUST CP Geeks](https://github.com/sust-cp-geeks)**

</div>
