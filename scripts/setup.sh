#!/usr/bin/env bash
# Food City — dev setup helper.
# Usage: ./scripts/setup.sh
#
# Verifies toolchain, installs dependencies, runs migrations.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

log() { printf '\033[1;36m[setup]\033[0m %s\n' "$*"; }
err() { printf '\033[1;31m[err]\033[0m %s\n' "$*" >&2; exit 1; }

# --- Toolchain checks ------------------------------------------------------
log "checking toolchain"

command -v cargo >/dev/null || err "Rust not installed. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
command -v node   >/dev/null || err "Node not installed. Install via: https://nodejs.org/"
command -v docker >/dev/null 2>&1 && HAVE_DOCKER=1 || HAVE_DOCKER=0

cargo --version
node --version

# --- .env files ------------------------------------------------------------
log "creating .env files if missing"
[ -f backend/.env ]  || cp backend/.env.example  backend/.env
[ -f frontend/.env ] || cp frontend/.env.example frontend/.env

# --- Backend deps ----------------------------------------------------------
log "fetching backend dependencies (this may take a few minutes on first run)"
( cd backend && cargo fetch )

# --- Frontend deps ---------------------------------------------------------
log "installing frontend dependencies"
( cd frontend && npm install )

# --- Database --------------------------------------------------------------
if [ "$HAVE_DOCKER" = "1" ]; then
  log "starting postgres + redis via docker compose"
  docker compose up -d postgres redis
  log "waiting for postgres to be ready"
  for i in $(seq 1 30); do
    if docker compose exec -T postgres pg_isready -U foodcity >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
  log "running migrations"
  DATABASE_URL="postgres://foodcity:foodcity@localhost:5432/foodcity" \
    ( cd backend && cargo run -- migrate || sqlx migrate run ) 2>/dev/null \
    || log "migrations will run automatically on backend startup"
else
  log "docker not found — assuming local postgres + redis"
  log "make sure postgres is running on localhost:5432 with db=foodcity user=foodcity pass=foodcity"
  log "and redis on localhost:6379"
fi

log "✓ setup complete"
log "next steps:"
log "  cd backend  && cargo run       # start API on :8080"
log "  cd frontend && npm run dev     # start Vite on :5173"
log "  open http://localhost:5173"
