# Contributing to Food City

Thanks for your interest in contributing! This guide covers the basics.

## Development setup

```bash
git clone <repo-url> food-city
cd food-city
./scripts/setup.sh        # installs deps, starts postgres+redis
docker compose up -d      # full stack
```

- Frontend: http://localhost:5173
- Backend: http://localhost:8080
- API docs: [`docs/API_CONTRACT.md`](docs/API_CONTRACT.md)

## Architecture

Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full system design.
Quick summary:

- **Backend** (`backend/`): Rust + Axum, layered as `api → services → db/repos → domain`
- **Frontend** (`frontend/`): React + Vite + TS + Tailwind, role-based routing
- **DB**: PostgreSQL with 5 migrations (schema + seed + perf indexes + promos)
- **Cache/Realtime**: Redis (pub/sub + GEO + rate limiting)

## Code style

### Backend (Rust)

- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets -- -D warnings` — must pass cleanly
- Follow the layering rules:
  - `api/` may only call `services/`
  - `services/` may call `db/repos/` + other services + `utils/`
  - `db/repos/` may only call `sqlx` + `domain/`
  - `domain/` has zero external deps beyond `serde`, `chrono`, `uuid`
- Use `AppResult<T>` for all fallible operations
- Errors: prefer `AppError::business_rule()` for domain violations, `AppError::validation()` for input issues

### Frontend (TypeScript + React)

- Run `npm run typecheck` — must pass
- Run `npm run lint` — must pass with 0 warnings
- Use functional components + hooks only (no class components except `ErrorBoundary`)
- Use TanStack Query for all server state (`useQuery` / `useMutation`)
- Use Zustand for client state (auth, cart, UI)
- Use `@/` alias for imports
- Add new types to `src/types/` or `src/api/*.api.ts`
- Every page should handle: loading, error, empty, and success states

## Pull request checklist

- [ ] Code compiles (`cargo check` + `tsc --noEmit`)
- [ ] Tests pass (`cargo test` + `npm run lint`)
- [ ] No new warnings introduced
- [ ] Documentation updated (if API changed)
- [ ] Migrations are reversible or additive (no destructive `DROP`/`ALTER`)
- [ ] New endpoints have ownership checks where appropriate

## Adding a new feature

1. **Backend:**
   - Add domain types to `src/domain/`
   - Add repo functions to `src/db/repos/`
   - Add service logic to `src/services/`
   - Add HTTP handlers to `src/api/v1/`
   - Wire routes in `src/api/v1/mod.rs`
   - Add migration to `backend/migrations/` (timestamp-prefixed)

2. **Frontend:**
   - Add types to `src/types/index.ts`
   - Add API client to `src/api/<feature>.api.ts`
   - Add hooks to `src/hooks/use<Feature>.ts`
   - Add pages to `src/pages/<role>/`
   - Add route to `src/router.tsx` with `RequireRole` guard
   - Add nav link to `src/components/layout/Header.tsx`

3. **Docs:**
   - Update `docs/API_CONTRACT.md` with new endpoints
   - Update `docs/ARCHITECTURE.md` if module structure changed
   - Add to `docs/RUNBOOK.md` if new ops procedures needed

## Testing

### Backend

```bash
cd backend
cargo test                    # run all
cargo test order_state        # run specific
cargo test -- --nocapture     # show println output
```

Tests live in `backend/tests/` (integration) or inline `#[cfg(test)]` modules (unit).

### Frontend

```bash
cd frontend
npm run typecheck
npm run lint
# Component tests TBD (Vitest setup planned)
```

## Database migrations

Migrations auto-run on backend startup. To add a new one:

```bash
# Create file with next timestamp
touch backend/migrations/$(date +%Y%m%d%H%M%S)_describe_change.sql
```

Rules:
- Always `IF NOT EXISTS` / `IF EXISTS` for idempotency
- Never `DROP COLUMN` directly — rename first, deploy, drop later
- Use `CREATE INDEX CONCURRENTLY` for large tables (run manually, not via migrate)
- Add down-migration notes in a comment

## Releasing

1. Merge PR to `main`
2. CI runs: fmt + clippy + tests (backend), typecheck + lint + build (frontend), docker build
3. Tag release: `git tag v0.2.0 && git push --tags`
4. Deploy via `docker compose -f docker-compose.prod.yml up -d` or `kubectl apply -f deploy/k8s/`

## Getting help

- Read `docs/AUDIT.md` for known gaps
- Read `docs/RUNBOOK.md` for ops procedures
- Read `docs/EDGE_CASES.md` for failure modes and mitigations
- Check `worklog.md` for recent agent activity
