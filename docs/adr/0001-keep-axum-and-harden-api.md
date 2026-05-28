# ADR 0001: Keep Axum And Harden API Boundaries

## Status

Accepted

## Context

The backend is Rust with Axum, SQLx, PostgreSQL, Google OAuth, cookie sessions,
CSV imports, and J-Quants integration. Axum is currently used in routing,
handlers, extractors, middleware, errors, tests, and OpenAPI annotations.

The security-sensitive behavior is mostly application-owned:

- Session cookie issuance and validation
- OAuth state verification
- CSRF origin checks
- CORS allowlist
- Rate limiting
- Request body limits
- Error response shape
- Log redaction for query strings

Replacing Axum would not materially improve security by itself. It would create
a large migration surface while preserving the same application risks.

## Decision

Keep Axum for now.

Invest in:

1. Security hardening on the current Axum implementation.
2. A RESTful `/api/v1` contract.
3. A framework-thin `app` layer over time, so handlers become adapters instead
   of business logic owners.

Do not replace Axum until there is a concrete risk, maintenance blocker, or
capability gap that cannot be addressed inside the current stack.

## Consequences

### Positive

- Avoids high-risk framework migration churn.
- Lets security work land in small, reviewable PRs.
- Keeps existing tests, OpenAPI generation, Fly.io deployment, and middleware
  behavior usable.
- Creates a path to future framework replacement by reducing Axum coupling
  gradually.

### Negative

- Axum types remain in HTTP-facing modules for now.
- REST redesign and internal boundary cleanup must be done incrementally.
- Some tests will still depend on `Router` and Axum extractors until the app
  layer is introduced.

## Guardrails

- Do not mix framework migration, API redesign, DB schema changes, and frontend
  rewrites in one PR.
- Preserve the current error envelope unless a versioned API explicitly changes
  it.
- Preserve `session_token` as an HTTP-only cookie.
- Treat `/api/v1` as the new contract and old routes as compatibility routes
  during migration.

