# Internship Application System API (Rust / Axum + SeaORM)

A high-performance REST API for managing internship and job opportunities and applicant submissions, written in **Rust** using **Axum**, **SeaORM**, **PostgreSQL**, **Argon2**, and **JWT**.

---

## Tech Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) (0.8)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **ORM & Migrations**: [SeaORM](https://www.sea-ql.org/SeaORM/) + `sea-orm-migration` (PostgreSQL via SQLx)
- **Admin Panel**: SeaORM Pro (`sea-orm-pro` configuration + interactive dashboard mounted at `/admin`)
- **Interactive Documentation**: [utoipa](https://github.com/juhaku/utoipa) + `utoipa-swagger-ui` mounted at `/docs`
- **Authentication**: JWT (`jsonwebtoken`) with custom Axum extractor & Role-Based Access Control (RBAC)
- **Password Hashing**: `argon2` with cryptographically secure salt generation
- **Error Handling**: `thiserror` with consistent JSON shape: `{ "detail": "..." }`
- **Configuration**: `dotenvy` + `config` from `.env`

---

## Project Structure

```
internship-api/
├── src/
│   ├── main.rs               # App entry point, tracing, router assembly & admin mount
│   ├── lib.rs                # Library interface exposing modules
│   ├── config.rs             # Environment configuration (dotenvy + config)
│   ├── db.rs                 # SeaORM connection pool & AppState
│   ├── errors.rs             # AppError enum with IntoResponse { "detail": "..." }
│   ├── entities/             # SeaORM entities mirroring models
│   │   ├── mod.rs
│   │   ├── user.rs           # User entity & UserRole enum (applicant/admin)
│   │   ├── opportunity.rs    # Opportunity entity, OpportunityType & Status
│   │   └── application.rs    # Application entity & ApplicationStatus
│   ├── migration/            # SeaORM migrations (table creation, indexes, foreign keys)
│   │   ├── mod.rs            # MigratorTrait implementation
│   │   └── m20240101_000001_create_tables.rs
│   ├── routes/               # Axum route definitions & OpenAPI spec
│   │   ├── mod.rs            # Router nesting & ApiDoc OpenAPI definition
│   │   ├── auth.rs           # /auth routes
│   │   ├── opportunities.rs  # /opportunities routes
│   │   └── applications.rs   # /applications routes
│   ├── handlers/             # Business logic handlers
│   │   ├── mod.rs
│   │   ├── auth.rs           # Register, login, password hash/verify, JWT
│   │   ├── opportunities.rs  # CRUD with admin protection & pagination
│   │   └── applications.rs   # Apply, status transitions & withdrawal logic
│   ├── schemas/              # Serde & Utoipa request/response models
│   │   ├── mod.rs
│   │   ├── auth.rs           # UserCreate, UserResponse, LoginRequest, Token
│   │   ├── opportunity.rs    # OpportunityCreate, OpportunityUpdate, Response
│   │   └── application.rs    # ApplicationCreate, StatusUpdate, Response
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs           # AuthenticatedUser & AdminUser extractors
│   └── admin.rs              # SeaORM Pro admin setup & dashboard UI
├── pro_admin/                # SeaORM Pro TOML configuration directory
│   ├── config.toml           # Site branding, theme, and navigation
│   ├── dashboard.toml        # Dashboard statistics cards and layout
│   └── raw_tables/           # Table schemas for users, opportunities, applications
│       ├── users.toml
│       ├── opportunities.toml
│       └── applications.toml
├── assets/
│   └── admin/                # Official SeaORM Pro React/Ant Design Pro SPA assets
├── tests/
│   └── api_tests.rs          # Independent unit & integration tests
├── Cargo.toml                # Pinned dependencies with feature flags
├── .env.example
└── README.md
```

---

## Quick Start

### 1. Prerequisites

- **Rust toolchain** (1.80+): `rustc --version`
- **PostgreSQL**: running locally or via Docker

```bash
# Create the database in PostgreSQL
createdb internship_db
# Or using psql:
# psql -c "CREATE DATABASE internship_db;"
```

### 2. Configure Environment

Copy `.env.example` to `.env` and set your credentials:

```bash
cp .env.example .env
```

Default variables in `.env`:
```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/internship_db
JWT_SECRET=super-secret-jwt-key-replace-in-production
JWT_EXPIRY_HOURS=24
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
```

### 3. Run the Application

The database migrations run **automatically** on application startup.

```bash
cargo run
```

The server will start listening at `http://localhost:8000`.

- **Swagger UI**: [http://localhost:8000/docs](http://localhost:8000/docs)
- **OpenAPI JSON Spec**: [http://localhost:8000/api-docs/openapi.json](http://localhost:8000/api-docs/openapi.json)
- **Admin Panel**: [http://localhost:8000/admin](http://localhost:8000/admin)
- **Admin Config JSON**: [http://localhost:8000/admin/config](http://localhost:8000/admin/config)

### 4. Running Tests

```bash
cargo test
```

---

## Creating an Admin User

You can create an admin user in two ways:

### Option A: Using the Admin Panel GUI
1. Register a standard user via `POST /auth/register`.
2. Open [http://localhost:8000/admin](http://localhost:8000/admin) in your browser.
3. Click on **Users** in the sidebar, find the user, and click **Promote to Admin**.

### Option B: Directly via SQL
```sql
UPDATE users SET role = 'admin' WHERE email = 'admin@example.com';
```

---

## Example curl Requests

### 1. Register a New User

```bash
curl -s -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Jane Doe",
    "email": "jane@example.com",
    "password": "secret123"
  }'
```

**Response (201 Created):**
```json
{
  "id": 1,
  "full_name": "Jane Doe",
  "email": "jane@example.com",
  "role": "applicant",
  "created_at": "2026-09-11T14:00:00+00:00"
}
```

---

### 2. Login (Obtain JWT Access Token)

Supports both JSON and standard OAuth2 form encoding (`application/x-www-form-urlencoded`):

```bash
curl -s -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "jane@example.com",
    "password": "secret123"
  }'
```

Or using form data:
```bash
curl -s -X POST http://localhost:8000/auth/login \
  -d "username=jane@example.com&password=secret123"
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOi...",
  "token_type": "bearer"
}
```

Save your token for subsequent requests:
```bash
export TOKEN="<paste-your-access-token-here>"
```

---

### 3. Create an Opportunity (Admin Only)

*(Requires user to have `admin` role)*

```bash
curl -s -X POST http://localhost:8000/opportunities \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Backend Engineering Intern",
    "description": "Build high-throughput async microservices with Rust & PostgreSQL",
    "company": "Acme Corp",
    "location": "Remote",
    "type": "internship"
  }'
```

**Response (201 Created):**
```json
{
  "id": 1,
  "title": "Backend Engineering Intern",
  "description": "Build high-throughput async microservices with Rust & PostgreSQL",
  "company": "Acme Corp",
  "location": "Remote",
  "type": "internship",
  "status": "open",
  "created_at": "2026-09-11T14:05:00+00:00",
  "updated_at": "2026-09-11T14:05:00+00:00"
}
```

---

### 4. List Open Opportunities (With Pagination)

```bash
curl -s -X GET "http://localhost:8000/opportunities?skip=0&limit=20" \
  -H "Authorization: Bearer $TOKEN"
```

**Response (200 OK):**
```json
[
  {
    "id": 1,
    "title": "Backend Engineering Intern",
    "description": "Build high-throughput async microservices with Rust & PostgreSQL",
    "company": "Acme Corp",
    "location": "Remote",
    "type": "internship",
    "status": "open",
    "created_at": "2026-09-11T14:05:00+00:00",
    "updated_at": "2026-09-11T14:05:00+00:00"
  }
]
```

---

### 5. Apply to an Opportunity

Enforces:
- Cannot apply to closed opportunities (`400 Bad Request`)
- Cannot apply to the same opportunity twice (`409 Conflict`)

```bash
curl -s -X POST http://localhost:8000/applications \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "opportunity_id": 1,
    "cover_letter": "I have hands-on experience with Rust, Axum, and SQL databases."
  }'
```

**Response (201 Created):**
```json
{
  "id": 1,
  "user_id": 1,
  "opportunity_id": 1,
  "cover_letter": "I have hands-on experience with Rust, Axum, and SQL databases.",
  "status": "pending",
  "applied_at": "2026-09-11T14:10:00+00:00",
  "updated_at": "2026-09-11T14:10:00+00:00"
}
```

---

### 6. View My Applications

```bash
curl -s -X GET http://localhost:8000/applications/me \
  -H "Authorization: Bearer $TOKEN"
```

---

### 7. Update Application Status (Admin Accept/Reject)

Admins can transition pending applications to `accepted` or `rejected`:

```bash
curl -s -X PATCH http://localhost:8000/applications/1/status \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "accepted"
  }'
```

---

### 8. Withdraw Application (Applicant)

Applicants can withdraw their pending applications:

```bash
curl -s -X PATCH http://localhost:8000/applications/1/status \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "withdrawn"
  }'
```

---

### 9. Delete (Withdraw) a Pending Application

```bash
curl -s -X DELETE http://localhost:8000/applications/1 \
  -H "Authorization: Bearer $TOKEN"
```

**Response (204 No Content)**

---

## License

MIT
