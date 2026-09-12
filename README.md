# Internship Application System

An API for managing internship and job postings, candidate applications, applicant profiles, and hiring workflows. It handles user lifecycle management, role-based authorization, rate-limited authentication, automated opportunity expiration, and asynchronous email notifications.

## Stack

- Framework: Axum 0.8 on Tokio
- ORM: SeaORM 2.0 with PostgreSQL migrations
- Database: PostgreSQL
- Auth: JWT (HMAC-SHA256) with Argon2id password hashing and SHA-256 refresh tokens
- Mail: Lettre with Askama HTML templates via Brevo SMTP
- Scheduler: tokio-cron-scheduler
- Docs: OpenAPI 3.0 / Swagger UI via Utoipa

## Getting Started Locally

```bash
git clone https://github.com/Penivera/dual-hq.git
cd dual-hq
cp .env.example .env
```

Configure `DATABASE_URL` in `.env`, create the database, and start the application:

```bash
createdb internship_db
cargo run
```

Database migrations run automatically on startup. If you prefer to apply migrations manually via SeaORM CLI before running:

```bash
sea-orm-cli migrate up
```

## Environment Variables

| Variable | Description | Required / Default |
| --- | --- | --- |
| DATABASE_URL | PostgreSQL connection string | Required (`postgresql://postgres:password@localhost:5432/internship_db`) |
| DB_MAX_CONNECTIONS | Maximum connections in the database pool | Optional (`10`) |
| DB_MIN_CONNECTIONS | Minimum idle connections retained in pool | Optional (`2`) |
| DB_CONNECT_TIMEOUT_SECS | Connection establishment timeout in seconds | Optional (`5`) |
| DB_IDLE_TIMEOUT_SECS | Max idle connection duration before closing | Optional (`600`) |
| JWT_SECRET | Secret key for signing access tokens | Optional (`super-secret-jwt-key-replace-in-production`) |
| JWT_EXPIRY_HOURS | Fallback access token expiration duration | Optional (`24`) |
| SERVER_HOST | Network interface to bind | Optional (`0.0.0.0`) |
| SERVER_PORT | Port to listen on | Optional (`8010`) |
| APP_BASE_URL | Base URL for links generated in email templates | Optional (`http://localhost:8010`) |
| ADMIN_EMAIL | Email address for initial bootstrapped admin | Optional (`admin@internship.local`) |
| ADMIN_PASSWORD | Password for initial bootstrapped admin | Optional (`admin`) |
| ADMIN_NAME | Display name for initial bootstrapped admin | Optional (`Admin User`) |
| SMTP_HOST | Brevo SMTP host | Optional (`smtp-relay.brevo.com`) |
| SMTP_PORT | Brevo SMTP port (587 STARTTLS, 465 TLS) | Optional (`587`) |
| SMTP_USER | Brevo SMTP account username | Optional (empty) |
| SMTP_PASSWORD | Brevo SMTP key or API password | Optional (empty) |
| SMTP_FROM_EMAIL | Outbound sender email address | Optional (empty) |
| SMTP_FROM_NAME | Outbound sender name displayed to recipients | Optional (`Peni Demo`) |
| SMTP_ENABLED | Enable or disable outbound mail delivery | Optional (`false`) |

## Roles

The application defines three roles: applicant, recruiter, and admin. Applicants self-register and start active immediately; they can manage their profile, browse opportunities, apply, and withdraw applications. Recruiters self-register in a pending status and cannot log in until an admin approves their account; once approved, recruiters post listings that go live immediately, edit or delete only their own postings, and review applications and candidate profiles submitted to their listings. Admins hold full system access: they approve pending recruiters, suspend or reactivate users, and manage categories.

## Creating an Admin User

The application seeds an initial admin account on startup using the `ADMIN_EMAIL`, `ADMIN_PASSWORD`, and `ADMIN_NAME` values from your `.env` file.

To create or update an admin from the CLI:

```bash
cargo run --bin seed -- admin@example.com SecretPass123 "Platform Admin"
```

To promote an existing user directly via PostgreSQL:

```sql
UPDATE users SET role = 'admin' WHERE email = 'admin@example.com';
```

## API Reference

Interactive OpenAPI / Swagger UI documentation is available locally at http://localhost:8010/docs and live at https://demo.peni.dev/docs. The API organizes endpoints into seven resource groups:

- Auth: Registration, login, token rotation, and logout with rate limiting
- Opportunities: Postings, composable filtering, recruiter ownership, and application counts
- Applications: Submissions, applicant withdrawals, status transitions, and recruiter reviews
- Categories: Taxonomy management and slug-based listing
- Profile: Applicant bios, validated resume URLs, and profile inspection
- Notifications: User in-app notifications and read receipts
- Admin: Recruiter approvals, user suspensions, reactivations, and dashboard metrics

## Example Requests

### Register as a Recruiter

```bash
curl -s -X POST http://localhost:8010/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Sarah Connor",
    "email": "sarah.connor@cyberdyne.io",
    "password": "Password123!",
    "role": "recruiter"
  }'
```

Response:

```json
{
  "id": 2,
  "full_name": "Sarah Connor",
  "email": "sarah.connor@cyberdyne.io",
  "role": "recruiter",
  "status": "pending",
  "is_verified": false,
  "created_at": "2026-09-12T10:00:00Z"
}
```

### Log In to Get Tokens

```bash
curl -s -X POST http://localhost:8010/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@internship.local",
    "password": "admin"
  }'
```

Response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzI2MTQ0MDAwfQ.k8D17v2f8...",
  "token_type": "bearer",
  "expires_in": 900,
  "refresh_token": "b4a8c9e1f2d3e4a5b6c7d8e9f0123456789abcdef0123456789abcdef0123456"
}
```

### Refresh an Access Token

```bash
curl -s -X POST http://localhost:8010/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "b4a8c9e1f2d3e4a5b6c7d8e9f0123456789abcdef0123456789abcdef0123456"
  }'
```

Response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzI2MTQ0OTAwfQ.m3F92q1z4...",
  "token_type": "bearer",
  "expires_in": 900,
  "refresh_token": "7f8e9d0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e"
}
```

### Logout

```bash
curl -s -X POST http://localhost:8010/auth/logout \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzI2MTQ0OTAwfQ.m3F92q1z4..." \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "7f8e9d0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e"
  }'
```

### Admin Approving a Recruiter

```bash
curl -s -X PATCH http://localhost:8010/admin/recruiters/2/approve \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzI2MTQ0MDAwfQ.k8D17v2f8..."
```

Response:

```json
{
  "id": 2,
  "full_name": "Sarah Connor",
  "email": "sarah.connor@cyberdyne.io",
  "role": "recruiter",
  "status": "active",
  "is_verified": false,
  "created_at": "2026-09-12T10:00:00Z"
}
```

### Admin Suspending a User

```bash
curl -s -X PATCH http://localhost:8010/admin/users/2/suspend \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzI2MTQ0MDAwfQ.k8D17v2f8..." \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Policy violation"
  }'
```

Response:

```json
{
  "id": 2,
  "full_name": "Sarah Connor",
  "email": "sarah.connor@cyberdyne.io",
  "role": "recruiter",
  "status": "suspended",
  "is_verified": false,
  "created_at": "2026-09-12T10:00:00Z"
}
```

## Deployment

The system is deployed live at https://demo.peni.dev. Swagger UI is accessible at `/docs`, and the SeaORM Pro admin panel is mounted at `/admin`.

## Background Jobs

An in-process scheduler managed by tokio-cron-scheduler runs an hourly job (`0 0 * * * *`) that queries for open opportunities whose deadline has passed and marks them closed. Any failures during execution are logged to stderr and do not interrupt API operations. The scheduler gracefully stops when the server receives a SIGTERM or SIGINT signal.

## Mail

Transactional mail is delivered through Brevo SMTP for email verification, welcome greetings, student opportunity broadcasts, application submissions, status updates, recruiter registration receipts, recruiter approvals, and account suspensions or reactivations. Deliveries run inside detached Tokio tasks (`tokio::spawn`), preventing network latency from delaying HTTP response times. If `BREVO_API_KEY` or SMTP credentials are not configured, or if `SMTP_ENABLED` is false, mail tasks log a warning and safely skip dispatch without returning errors to callers.
