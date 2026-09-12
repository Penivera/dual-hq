# Internship Application System API (Rust / Axum + SeaORM)

A high-performance REST API for managing internship and job opportunities and applicant submissions, written in **Rust** using **Axum**, **SeaORM**, **PostgreSQL**, **Argon2**, and **JWT**.

---

## Tech Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) (0.8)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **ORM & Migrations**: [SeaORM](https://www.sea-ql.org/SeaORM/) + `sea-orm-migration` (PostgreSQL via SQLx)
- **Admin Panel**: SeaORM Pro (`sea-orm-pro` configuration + interactive dashboard mounted at `/admin`)
- **GraphQL**: [async-graphql](https://github.com/async-graphql/async-graphql) schema and Playground mounted at `/graphql` and `/playground`
- **Interactive Documentation**: [utoipa](https://github.com/juhaku/utoipa) + `utoipa-swagger-ui` mounted at `/docs`
- **Authentication**: JWT (`jsonwebtoken`) with custom Axum extractor & Role-Based Access Control (RBAC)
- **Password Hashing**: `argon2` (Argon2id, V0x13, m_cost: 19456, t_cost: 2, p_cost: 1)
- **Email Notifications**: [lettre](https://github.com/lettre/lettre) + [askama](https://github.com/rinja-rs/askama) HTML templates (non-blocking async background delivery via `tokio::spawn`)
- **Resilience & Compression**: [tower-http](https://github.com/tower-rs/tower-http) Gzip & Brotli compression + 30-second request timeout
- **Error Handling**: `thiserror` with consistent JSON shape: `{ "detail": "..." }`
- **Configuration**: `dotenvy` + `config` from `.env`

---

## Project Structure

```
internship-api/
├── src/
│   ├── main.rs               # App entry point, tracing, compression & timeout layers, router assembly
│   ├── lib.rs                # Library interface exposing modules
│   ├── config.rs             # Environment configuration (dotenvy + config)
│   ├── db.rs                 # SeaORM connection pool & AppState
│   ├── email.rs              # SMTP email dispatcher with askama templates & background tokio tasks
│   ├── errors.rs             # AppError enum with IntoResponse { "detail": "..." }
│   ├── graphql.rs            # async-graphql schema, queries, and mutations
│   ├── entities/             # SeaORM entities mirroring models
│   │   ├── mod.rs
│   │   ├── user.rs           # User entity & UserRole enum (applicant/admin)
│   │   ├── opportunity.rs    # Opportunity entity, OpportunityType & Status
│   │   └── application.rs    # Application entity & ApplicationStatus
│   ├── migration/            # SeaORM migrations (table creation, indexes, foreign keys, constraints)
│   │   ├── mod.rs            # MigratorTrait implementation
│   │   ├── m20240101_000001_create_tables.rs
│   │   ├── m20240101_000002_add_opportunity_indexes.rs
│   │   └── m20240101_000003_enforce_constraints.rs
│   ├── routes/               # Axum route definitions & OpenAPI spec
│   │   ├── mod.rs            # Router nesting & ApiDoc OpenAPI definition
│   │   ├── auth.rs           # /auth routes
│   │   ├── opportunities.rs  # /opportunities routes
│   │   └── applications.rs   # /applications routes
│   ├── handlers/             # Business logic handlers
│   │   ├── mod.rs
│   │   ├── auth.rs           # Register, login, password hash/verify, JWT
│   │   ├── opportunities.rs  # Paginated queries & CRUD with admin protection
│   │   └── applications.rs   # Apply, status transitions, single-query join & withdrawal logic
│   ├── schemas/              # Serde & Utoipa request/response models
│   │   ├── mod.rs
│   │   ├── auth.rs           # UserCreate, UserResponse, LoginRequest, Token
│   │   ├── opportunity.rs    # OpportunityCreate, OpportunityFullUpdate, Response, PaginationQuery
│   │   └── application.rs    # ApplicationCreate, StatusUpdate, ApplicationDetailResponse
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs           # AuthenticatedUser, AdminUser & ApplicantUser extractors
│   └── admin.rs              # SeaORM Pro admin setup & dashboard UI
├── templates/                # Askama HTML email templates
│   ├── welcome.html
│   ├── new_opportunity.html
│   ├── application_submitted.html
│   └── application_status.html
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
│   ├── api_tests.rs          # API integration tests
│   └── email_tests.rs        # Email template rendering & SMTP graceful degradation tests
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
# PostgreSQL database connection string
DATABASE_URL=postgresql://postgres:password@localhost:5432/internship_db

# Database Connection Pool Configuration
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2
DB_CONNECT_TIMEOUT_SECS=5
DB_IDLE_TIMEOUT_SECS=600

# JWT configuration
JWT_SECRET=super-secret-jwt-key-replace-in-production
JWT_EXPIRY_HOURS=24

# Server network settings
SERVER_HOST=0.0.0.0
SERVER_PORT=8010
APP_BASE_URL=http://localhost:8010

# Initial Administrator Seed Settings
ADMIN_EMAIL=admin@internship.local
ADMIN_PASSWORD=admin
ADMIN_NAME="Admin User"

# SMTP Mailing Configuration (Brevo / Sendinblue)
SMTP_HOST=smtp-relay.brevo.com
SMTP_PORT=587
SMTP_USER=your-smtp-login-here
SMTP_PASSWORD=your-smtp-key-here
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME="Peni Demo"
SMTP_ENABLED=true
```

### 3. Run the Application

The database migrations run **automatically** on application startup.

```bash
cargo run
```

The server will start listening at `http://localhost:8010`.

- **Swagger UI**: [http://localhost:8010/docs](http://localhost:8010/docs)
- **OpenAPI JSON Spec**: [http://localhost:8010/api-docs/openapi.json](http://localhost:8010/api-docs/openapi.json)
- **GraphQL Playground**: [http://localhost:8010/playground](http://localhost:8010/playground)
- **Admin Panel**: [http://localhost:8010/admin](http://localhost:8010/admin)
- **Admin Config JSON**: [http://localhost:8010/admin/config](http://localhost:8010/admin/config)

### 4. Running Tests

```bash
cargo test
```

---

## Creating an Admin User

You can create an admin user in three ways:

### Option A: Automatic Seeding on Startup
Set `ADMIN_EMAIL`, `ADMIN_PASSWORD`, and `ADMIN_NAME` in your `.env`. When the application starts up and runs migrations, it automatically ensures this admin user exists.

### Option B: Using the Admin Panel GUI
1. Register a standard user via `POST /auth/register`.
2. Open [http://localhost:8010/admin](http://localhost:8010/admin) in your browser.
3. Click on **Users** in the sidebar, find the user, and click **Promote to Admin**.

### Option C: Directly via SQL
```sql
UPDATE users SET role = 'admin' WHERE email = 'admin@internship.local';
```

---

## Example API Requests

### 1. Register a New User

```bash
curl -s -X POST http://localhost:8010/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Jane Doe",
    "email": "jane@example.com",
    "password": "secretPassword123"
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
*(Asynchronously dispatches a welcome email via background Tokio task if SMTP is configured).*

---

### 2. Login (Obtain JWT Access Token)

Supports both JSON and standard OAuth2 form encoding (`application/x-www-form-urlencoded`):

```bash
curl -s -X POST http://localhost:8010/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "jane@example.com",
    "password": "secretPassword123"
  }'
```

Or using form data:
```bash
curl -s -X POST http://localhost:8010/auth/login \
  -d "username=jane@example.com&password=secretPassword123"
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

### 3. Opportunities

#### A. Create an Opportunity (Admin Only)
Requires user to have `admin` role. Returns `422 Unprocessable Entity` if any required fields are empty or whitespace.

```bash
curl -s -X POST http://localhost:8010/opportunities \
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
*(Asynchronously dispatches new opportunity notification emails to registered applicants).*

#### B. List Open Opportunities (Paginated)
Supports `page` (default 1) and `per_page` (default 20, max 100), as well as backwards-compatible `skip` and `limit`. Only returns `open` opportunities.

```bash
curl -s -X GET "http://localhost:8010/opportunities?page=1&per_page=20" \
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

#### C. Get Opportunity by ID
Returns `200 OK` or `404 Not Found` (`{ "detail": "Opportunity not found" }`).

```bash
curl -s -X GET http://localhost:8010/opportunities/1 \
  -H "Authorization: Bearer $TOKEN"
```

#### D. Full Update Opportunity (Admin Only)
Performs a full update. Returns `200 OK` or `422 Unprocessable Entity` if any required fields are empty or whitespace.

```bash
curl -s -X PUT http://localhost:8010/opportunities/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Senior Backend Engineering Intern",
    "description": "Lead async microservices with Rust, SeaORM & Axum",
    "company": "Acme Corp",
    "location": "Remote (Global)",
    "type": "internship",
    "status": "open"
  }'
```

#### E. Delete Opportunity (Admin Only)
```bash
curl -s -i -X DELETE http://localhost:8010/opportunities/1 \
  -H "Authorization: Bearer $TOKEN"
```
**Response (204 No Content)**

---

### 4. Applications

#### A. Apply to an Opportunity (Applicant Only)
Enforces:
- Only users with `applicant` role can apply (`403 Forbidden` for admin)
- Cannot apply to closed opportunities (`400 Bad Request`)
- Cannot apply to the same opportunity twice (`409 Conflict`)
- Cover letter cannot be empty (`422 Unprocessable Entity`)

```bash
curl -s -X POST http://localhost:8010/applications \
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
*(Asynchronously dispatches application submission confirmation email to applicant).*

#### B. View My Applications
Fetches all applications submitted by the authenticated user joined with opportunity title and company name in a single SQL query (`find_also_related`).

```bash
curl -s -X GET http://localhost:8010/applications/me \
  -H "Authorization: Bearer $TOKEN"
```

**Response (200 OK):**
```json
[
  {
    "id": 1,
    "user_id": 1,
    "opportunity_id": 1,
    "opportunity_title": "Senior Backend Engineering Intern",
    "company": "Acme Corp",
    "cover_letter": "I have hands-on experience with Rust, Axum, and SQL databases.",
    "status": "pending",
    "applied_at": "2026-09-11T14:10:00+00:00",
    "updated_at": "2026-09-11T14:10:00+00:00"
  }
]
```

#### C. Get Application by ID
Only accessible by the application owner or an administrator. Returns `403 Forbidden` otherwise, or `404 Not Found`.

```bash
curl -s -X GET http://localhost:8010/applications/1 \
  -H "Authorization: Bearer $TOKEN"
```

#### D. Update Application Status
Enforces strict state transitions:
- **Admin**: Can transition `pending` applications to `accepted` or `rejected`. Invalid transitions return `422 Unprocessable Entity`. Asynchronously sends a status update email to the applicant.
- **Applicant**: Can transition `pending` applications to `withdrawn`. Non-pending applications or invalid status targets return `422 Unprocessable Entity`.

```bash
# Admin accepting an application:
curl -s -X PATCH http://localhost:8010/applications/1/status \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "accepted"
  }'
```

```bash
# Applicant withdrawing their pending application:
curl -s -X PATCH http://localhost:8010/applications/1/status \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "withdrawn"
  }'
```

#### E. Delete Application (Applicant Only)
Applicants can delete their own application only while its status is still `pending`. If the application is already accepted, rejected, or withdrawn, it returns `422 Unprocessable Entity`.

```bash
curl -s -i -X DELETE http://localhost:8010/applications/1 \
  -H "Authorization: Bearer $TOKEN"
```

**Response (204 No Content)**

---

## License

MIT
