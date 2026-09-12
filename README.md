# Internship Application System

A REST API for managing internship postings and applicant submissions. It provides role-based access control for applicants and administrators, application status tracking with strict transition validation, asynchronous transactional emails for workflow events, and an administrative panel for system management.

The deployed version is live at https://demo.peni.dev.

## Stack

- Framework: Axum 0.8 on Tokio
- ORM: SeaORM with sea-orm-migration (PostgreSQL)
- Database: PostgreSQL
- Auth: JWT (HMAC-SHA256) with Argon2id password hashing
- Mail: Lettre with Askama HTML templates (Brevo SMTP)
- Docs: OpenAPI 3.0 / Swagger UI via Utoipa

## Getting Started Locally

```bash
git clone https://github.com/Penivera/dual-hq.git
cd dual-hq
cp .env.example .env
createdb internship_db
cargo run
```

Migrations execute automatically on startup. The server listens on `http://0.0.0.0:8010`.

To run the test suite:

```bash
cargo test
```

## Environment Variables

| Variable | Description | Status | Default |
| --- | --- | --- | --- |
| DATABASE_URL | PostgreSQL connection string | Required | postgresql://postgres:password@localhost:5432/internship_db |
| DB_MAX_CONNECTIONS | Maximum connection pool size | Optional | 10 |
| DB_MIN_CONNECTIONS | Minimum idle connections in pool | Optional | 2 |
| DB_CONNECT_TIMEOUT_SECS | Database connection timeout in seconds | Optional | 5 |
| DB_IDLE_TIMEOUT_SECS | Idle connection timeout in seconds | Optional | 600 |
| JWT_SECRET | Secret key for signing and verifying tokens | Optional | super-secret-jwt-key-replace-in-production |
| JWT_EXPIRY_HOURS | Token lifetime in hours | Optional | 24 |
| SERVER_HOST | Host interface to bind | Optional | 0.0.0.0 |
| SERVER_PORT | TCP port to listen on | Optional | 8010 |
| APP_BASE_URL | Base application URL used in email template links | Optional | http://localhost:8010 |
| ADMIN_EMAIL | Email of the pre-seeded admin user | Optional | admin@internship.local |
| ADMIN_PASSWORD | Password of the pre-seeded admin user | Optional | admin |
| ADMIN_NAME | Display name of the pre-seeded admin user | Optional | Admin User |
| SMTP_HOST | SMTP server hostname | Optional | smtp-relay.brevo.com |
| SMTP_PORT | SMTP port (587 for STARTTLS, 465 for TLS) | Optional | 587 |
| SMTP_USER | SMTP login username | Optional | (empty) |
| SMTP_PASSWORD | SMTP password or Brevo API/SMTP key | Optional | (empty) |
| SMTP_FROM_EMAIL | Outgoing sender email address | Optional | (empty) |
| SMTP_FROM_NAME | Outgoing sender display name | Optional | Peni Demo |
| SMTP_ENABLED | Master switch for outbound transactional mail | Optional | false |

## Creating an Admin User

The server checks for an admin user on startup. If no admin exists, it creates one using `ADMIN_EMAIL`, `ADMIN_PASSWORD`, and `ADMIN_NAME` from `.env`.

To create or promote an admin explicitly via CLI:

```bash
cargo run --bin seed -- admin@example.com SecretPass123 "Platform Admin"
```

Or update an existing user directly in PostgreSQL:

```sql
UPDATE users SET role = 'admin' WHERE email = 'john.doe@example.com';
```

## API Overview

| Method | Path | Auth | Role | Description |
| --- | --- | --- | --- | --- |
| POST | /auth/register | No | Public | Register a new user account |
| POST | /auth/login | No | Public | Authenticate user and receive JWT bearer token |
| GET | /auth/verify | No | Public | Verify account email via query token |
| POST | /auth/verify | No | Public | Verify account email via JSON payload |
| POST | /auth/resend-verification | No | Public | Resend email verification link |
| GET | /opportunities | Yes | Any | List open opportunities with pagination (page, per_page) |
| POST | /opportunities | Yes | Admin | Create a new opportunity (broadcasts email to applicants) |
| GET | /opportunities/{id} | Yes | Any | Fetch single opportunity by ID |
| PUT | /opportunities/{id} | Yes | Admin | Full update of an opportunity |
| DELETE | /opportunities/{id} | Yes | Admin | Delete an opportunity |
| POST | /applications | Yes | Applicant | Apply to an open opportunity |
| GET | /applications | Yes | Admin | List all applications across the platform |
| GET | /applications/me | Yes | Applicant | List current user applications with opportunity details joined |
| GET | /applications/{id} | Yes | Owner/Admin | Fetch application details |
| PATCH | /applications/{id}/status | Yes | Owner/Admin | Transition application status |
| DELETE | /applications/{id} | Yes | Applicant | Delete a pending application |
| GET | /health | No | Public | Service health and database connectivity check |

Interactive Swagger documentation is available at `/docs` and OpenAPI JSON at `/api-docs/openapi.json`.
The SeaORM Pro admin dashboard is available at `/admin`.
GraphQL schema and playground are available at `/graphql` and `/playground`.

## Example Requests

### Register

```bash
curl -s -X POST http://localhost:8010/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "John Doe",
    "email": "john.doe@example.com",
    "password": "Password123!"
  }'
```

Response:
```json
{
  "id": 2,
  "full_name": "John Doe",
  "email": "john.doe@example.com",
  "role": "applicant",
  "created_at": "2026-09-12T01:30:00Z"
}
```

### Login

```bash
curl -s -X POST http://localhost:8010/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "password": "Password123!"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIyIiwicm9sZSI6ImFwcGxpY2FudCIsImV4cCI6MTc4OTEwMDAwMH0.K3-4-8b_G8u7eG",
  "token_type": "bearer"
}
```

### Create an Opportunity (Admin)

```bash
curl -s -X POST http://localhost:8010/opportunities \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzg5MTAwMDAwfQ.abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Systems Software Engineering Intern",
    "description": "Design and build high-throughput backend services using Rust and PostgreSQL.",
    "company": "DualHQ",
    "location": "Remote",
    "type": "internship"
  }'
```

Response:
```json
{
  "id": 1,
  "title": "Systems Software Engineering Intern",
  "description": "Design and build high-throughput backend services using Rust and PostgreSQL.",
  "company": "DualHQ",
  "location": "Remote",
  "type": "internship",
  "status": "open",
  "created_at": "2026-09-12T01:32:00Z",
  "updated_at": "2026-09-12T01:32:00Z"
}
```

### Apply to an Opportunity

```bash
curl -s -X POST http://localhost:8010/applications \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIyIiwicm9sZSI6ImFwcGxpY2FudCIsImV4cCI6MTc4OTEwMDAwMH0.K3-4-8b_G8u7eG" \
  -H "Content-Type: application/json" \
  -d '{
    "opportunity_id": 1,
    "cover_letter": "I have hands-on experience building asynchronous backend APIs in Rust and working with relational databases."
  }'
```

Response:
```json
{
  "id": 1,
  "user_id": 2,
  "opportunity_id": 1,
  "cover_letter": "I have hands-on experience building asynchronous backend APIs in Rust and working with relational databases.",
  "status": "pending",
  "applied_at": "2026-09-12T01:35:00Z",
  "updated_at": "2026-09-12T01:35:00Z"
}
```

### Check Applications

```bash
curl -s -X GET http://localhost:8010/applications/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIyIiwicm9sZSI6ImFwcGxpY2FudCIsImV4cCI6MTc4OTEwMDAwMH0.K3-4-8b_G8u7eG"
```

Response:
```json
[
  {
    "id": 1,
    "user_id": 2,
    "opportunity_id": 1,
    "opportunity_title": "Systems Software Engineering Intern",
    "company": "DualHQ",
    "cover_letter": "I have hands-on experience building asynchronous backend APIs in Rust and working with relational databases.",
    "status": "pending",
    "applied_at": "2026-09-12T01:35:00Z",
    "updated_at": "2026-09-12T01:35:00Z"
  }
]
```

### Update Application Status

```bash
curl -s -X PATCH http://localhost:8010/applications/1/status \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjoxNzg5MTAwMDAwfQ.abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "accepted"
  }'
```

Response:
```json
{
  "id": 1,
  "user_id": 2,
  "opportunity_id": 1,
  "cover_letter": "I have hands-on experience building asynchronous backend APIs in Rust and working with relational databases.",
  "status": "accepted",
  "applied_at": "2026-09-12T01:35:00Z",
  "updated_at": "2026-09-12T01:40:00Z"
}
```

## Deployment

The application is deployed at https://demo.peni.dev.

- Swagger UI is live at https://demo.peni.dev/docs
- SeaORM Pro admin panel is live at https://demo.peni.dev/admin
- Health check is accessible at https://demo.peni.dev/health

## Mail

Transactional email is handled via Brevo over SMTP. Emails are sent for five triggers: email verification requests, welcome confirmation on verification, opportunity creation notifications sent to students, application submission confirmations, and application status updates (accepted/rejected). Delivery runs in detached Tokio tasks (`tokio::spawn`), so network latency to the SMTP server does not affect HTTP response times. If `BREVO_API_KEY` or SMTP credentials (`SMTP_USER`, `SMTP_PASSWORD`) are not provided, or `SMTP_ENABLED=false`, the email service logs an info trace and safely skips delivery without failing the HTTP request.
