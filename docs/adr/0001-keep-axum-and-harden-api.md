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

## Architecture Stance

This ADR also fixes the overall architecture style the project commits to. It
is intentionally pragmatic and does not require a framework migration.

### Backend: Modular Monolith with Thin HTTP Adapter Layer

- Single Rust/Axum binary, organized by domain modules
  (`handlers / services / models / extractors / middleware / routes / errors / state`).
- Handlers stay thin: parse request, call services, map errors to the v1
  envelope. Business rules live in `services`.
- Cross-module calls go through service functions, not through HTTP.
- SQLx is used directly in services. The internal boundary is the service function signature, not a trait.

### Frontend: Feature-first Architecture with Shared UI Components

- Code is grouped by feature (`features/auth`, `features/stock`,
  `features/receipt`, `features/assetBalance`, `features/marketData`,
  `features/dividendPerShare`).
- Cross-feature UI primitives stay in `components/` (Atomic Design) and
  generic utilities stay in `lib/`.
- Feature API wrappers use the generated API types in `src/generated/api.ts`,
  driven by `docs/openapi.json`.

### What we are not adopting

- **Full hexagonal / full clean architecture.** The cost of ports, adapters,
  use-case objects, and entity/DTO duplication is not justified at this size.
- **A Repository trait per aggregate.** SQLx queries inside service functions
  are simpler, faster to evolve, and easier to test against a real database.
- **A DI container.** `AppState` and function arguments already cover
  dependency wiring without runtime resolution magic.
- **Wholesale `domain` / `application` / `infrastructure` split.** Domain
  rules live next to the service that owns them; we add structure only when a
  concrete pain forces it.

### Where boundaries are drawn

- **HTTP boundary**: handlers + extractors. Axum types stop here.
- **Service boundary**: `services::<domain>` functions own business rules,
  validation, and DB access for that domain.
- **Persistence boundary**: SQLx queries live inside service modules; no
  generic repository abstraction.
- **Frontend feature boundary**: each feature owns its hooks, components, and
  API calls; shared concerns are promoted to `components/` or `lib/` only
  when reused.

### When to revisit

- A second transport (gRPC, queue worker) starts reusing the same business
  rules.
- A domain grows to the point where a single service module becomes hard to
  review.
- Tests start needing to fake the database at a granularity SQLx fixtures
  cannot cover.

Until one of those triggers fires, prefer adding clarity inside the current
structure over introducing new layers.

