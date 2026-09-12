# Internship Application System

A REST API for managing internship opportunities, applications, and hiring workflows. It provides role-based access control for applicants, recruiters, and administrators, account lifecycle management, applicant profiles with resume links, opportunity categorization and search filters, background auto-expiry via Tokio Cron Scheduler, rate-limited authentication with token rotation, in-app notifications, and asynchronous transactional emails via Brevo SMTP.

The deployed version is live at https://demo.peni.dev.

## Stack

- Framework: Axum 0.8 on Tokio
- ORM: SeaORM 2.0 with sea-orm-migration (PostgreSQL)
- Database: PostgreSQL
- Auth: JWT (HMAC-SHA256) with Argon2id password hashing, SHA-256 refresh tokens
- Rate Limiting: tower_governor (governor)
- Scheduling: tokio-cron-scheduler
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
| JWT_SECRET | Secret key for signing and verifying access tokens | Optional | super-secret-jwt-key-replace-in-production |
| JWT_EXPIRY_HOURS | Token lifetime fallback in hours | Optional | 24 |
| SERVER_HOST | Host interface to bind | Optional | 0.0.0.0 |
| SERVER_PORT | TCP port to listen on | Optional | 8010 |
| APP_BASE_URL | Base application URL used in email template links | Optional | http://localhost:8010 |
| ADMIN_EMAIL | Email of the pre-seeded admin user | Optional | admin@internship.local |
| ADMIN_PASSWORD | Password of the pre-seeded admin user | Optional | admin |
| ADMIN_NAME | Display name of the pre-seeded admin user | Optional | Admin User |
| SMTP_HOST | SMTP server hostname | Optional | smtp-relay.brevo.com |
| SMTP_PORT | SMTP port (587 for STARTTLS, 465 for TLS) | Optional | 587 |
| SMTP_USER | SMTP login username | Optional | (empty) |
| SMTP_PASSWORD | SMTP password or Brevo API key | Optional | (empty) |
| SMTP_FROM_EMAIL | Outgoing sender email address | Optional | (empty) |
| SMTP_FROM_NAME | Outgoing sender display name | Optional | Peni Demo |
| SMTP_ENABLED | Master switch for outbound transactional mail | Optional | false |

## User Roles and Account Lifecycle

The system supports three distinct user roles:

1. Applicant: Default registration role. Accounts are immediately active. Applicants can browse open opportunities, submit and track applications, manage their personal profile and CV link, and receive updates when application statuses change.
2. Recruiter: Self-registers with role `recruiter`. Newly created recruiter accounts start in `pending` status and cannot log in until approved by an administrator. Once active, recruiters can post opportunities (which go live immediately), modify or delete only their own listings, review applications submitted to their listings (including applicant bio and CV link), and update candidate status.
3. Admin: Full platform administrator. Can approve pending recruiters, suspend or unsuspend users, create and delete categories, and inspect platform metrics.

User accounts have one of three lifecycle statuses: `pending`, `active`, or `suspended`. Login attempts by pending or suspended users return 403 Forbidden with descriptive error messages. When an admin suspends a user, all of the user's active refresh tokens are immediately revoked.

## Authentication and Token Refresh

- Access Tokens: Signed JWTs valid for 15 minutes.
- Refresh Tokens: Cryptographically random 256-bit tokens valid for 7 days, stored in hashed format (SHA-256) in the database.
- Token Rotation: Calling `POST /auth/refresh` verifies the submitted token, revokes it immediately, and issues a fresh access token and refresh token pair.
- Logout: Calling `POST /auth/logout` revokes the provided refresh token or active session tokens.

## Rate Limiting

Endpoints vulnerable to abuse are rate-limited per IP using `tower_governor`:

- `POST /auth/login`: 10 requests per minute per IP
- `POST /auth/register`: 10 requests per minute per IP
- `POST /auth/refresh`: 20 requests per minute per IP

When a client exceeds the quota, the API responds with HTTP 429 Too Many Requests:

```json
{
  "detail": "too many requests, slow down"
}
```

GET endpoints remain unthrottled.

## Auto-Expiry Background Job

An hourly cron job runs via `tokio-cron-scheduler`. At the top of every hour (`0 0 * * * *`), the job finds all open opportunities whose deadline has passed (`deadline < NOW()`), marks their status as `closed`, and fires an in-app notification to the recruiter who created the listing. The scheduler logs the count of expired records and terminates cleanly on SIGTERM or SIGINT.

## API Overview

| Method | Path | Auth | Role | Description |
| --- | --- | --- | --- | --- |
| POST | /auth/register | No | Public | Register applicant (active) or recruiter (pending) |
| POST | /auth/login | No | Public | Log in with credentials; returns 15m access token and 7d refresh token |
| POST | /auth/refresh | No | Public | Rotate refresh token for a new token pair |
| POST | /auth/logout | Yes/Opt | Public | Revoke refresh tokens |
| GET | /auth/verify | No | Public | Verify email address via query token |
| POST | /auth/resend-verification | No | Public | Resend verification email |
| GET | /opportunities | Yes/Opt | Any | List opportunities with composable filters (`search`, `type`, `location`, `category`) |
| POST | /opportunities | Yes | Recruiter/Admin | Create opportunity listing (open by default, tracks `created_by`) |
| GET | /opportunities/{id} | Yes/Opt | Any | Retrieve single opportunity (`application_count` visible to owner/admin only) |
| PUT | /opportunities/{id} | Yes | Recruiter/Admin | Update opportunity (recruiters restricted to own listings) |
| DELETE | /opportunities/{id} | Yes | Recruiter/Admin | Delete opportunity (recruiters restricted to own listings) |
| GET | /categories | No | Public | List all opportunity categories |
| POST | /categories | Yes | Admin | Create category with auto-generated slug |
| DELETE | /categories/{id} | Yes | Admin | Delete category by ID |
| GET | /profile/me | Yes | Applicant | Get current user profile (creates default empty profile if none exists) |
| PUT | /profile/me | Yes | Applicant | Update user bio and validate CV URL |
| GET | /profile/{user_id} | Yes | Recruiter/Admin | View applicant profile details |
| POST | /applications | Yes | Applicant | Submit application to an open listing |
| GET | /applications | Yes | Recruiter/Admin | List applications (recruiters view only applications to their listings) |
| GET | /applications/me | Yes | Applicant | List current user applications with opportunity details |
| GET | /applications/{id} | Yes | Owner/Admin | Get application details (including applicant bio and CV link) |
| PATCH | /applications/{id}/status | Yes | Owner/Admin | Update candidate status (accepted, rejected, withdrawn) |
| DELETE | /applications/{id} | Yes | Applicant | Delete pending application |
| GET | /notifications | Yes | Any | List user notifications (unread first, then newest first) |
| PATCH | /notifications/{id}/read | Yes | Any | Mark notification as read |
| PATCH | /notifications/read-all | Yes | Any | Mark all user notifications as read |
| GET | /admin/recruiters/pending | Yes | Admin | List pending recruiter accounts awaiting approval |
| PATCH | /admin/recruiters/{id}/approve | Yes | Admin | Approve recruiter account and fire welcome email |
| PATCH | /admin/users/{id}/suspend | Yes | Admin | Suspend user, revoke all refresh tokens, fire notice email |
| PATCH | /admin/users/{id}/unsuspend | Yes | Admin | Reactivate user account and fire reactivation email |
| GET | /health | No | Public | Service health and database connectivity check |

Interactive Swagger documentation is available at `/docs` and OpenAPI JSON at `/api-docs/openapi.json`.
The SeaORM Pro admin dashboard is available at `/admin`.
GraphQL schema and playground are available at `/graphql` and `/playground`.

## Example Requests

### Register as a Recruiter

```bash
curl -s -X POST http://localhost:8010/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Jane Recruiter",
    "email": "jane@techcorp.com",
    "password": "Password123!",
    "role": "recruiter"
  }'
```

Response:
```json
{
  "id": 5,
  "full_name": "Jane Recruiter",
  "email": "jane@techcorp.com",
  "role": "recruiter",
  "status": "pending",
  "is_verified": false,
  "created_at": "2026-09-12T10:00:00Z"
}
```

### Log in (Receiving Token Pair)

```bash
curl -s -X POST http://localhost:8010/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "applicant@example.com",
    "password": "Password123!"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "8a3f9e2...",
  "token_type": "bearer",
  "expires_in": 900
}
```

### Rotate Refresh Token

```bash
curl -s -X POST http://localhost:8010/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "8a3f9e2..."
  }'
```

### Update Applicant Profile

```bash
curl -s -X PUT http://localhost:8010/profile/me \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "bio": "Systems programmer interested in high-performance backend infrastructure in Rust.",
    "cv_url": "https://cdn.example.com/resumes/john_doe.pdf"
  }'
```

### Filter Opportunities

```bash
curl -s -X GET "http://localhost:8010/opportunities?category=engineering&type=internship&location=remote&search=systems"
```

## Mail Service

Transactional emails are dispatched via Brevo SMTP using Askama HTML templates. Email events include:

- Email verification requests for new applicants
- Welcome notification upon email verification
- Notification of new opportunity listings broadcast to registered applicants
- Application submission confirmation
- Candidate status update (accepted or rejected)
- Recruiter registration received notice
- Recruiter account approval notice
- User suspension notice
- User reactivation notice

All email dispatch calls run inside asynchronous Tokio tasks (`tokio::spawn`), preventing network latency to Brevo from delaying HTTP response times.
