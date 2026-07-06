#!/usr/bin/env bash
# Restore PostgreSQL database from a backup file.
# Usage: ./scripts/restore-db.sh <backup-file>

set -euo pipefail

BACKUP="${1:-}"
if [ -z "$BACKUP" ]; then
  echo "Usage: $0 <backup-file>"
  exit 1
fi

if [ ! -f "$BACKUP" ]; then
  echo "Backup file not found: $BACKUP"
  exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

if [ -f .env.prod ]; then
  set -a
  . .env.prod
  set +a
fi

: "${POSTGRES_USER:?POSTGRES_USER not set}"
: "${POSTGRES_PASSWORD:?POSTGRES_PASSWORD not set}"
: "${POSTGRES_DB:=foodcity}"

PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"

read -p "This will DROP and RESTORE database '${POSTGRES_DB}' from ${BACKUP}. Continue? (yes/no) " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
  echo "Aborted."
  exit 1
fi

echo "Dropping and recreating ${POSTGRES_DB}..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$PGHOST" -p "$PGPORT" -U "$POSTGRES_USER" -d postgres <<SQL
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${POSTGRES_DB}';
DROP DATABASE IF EXISTS "${POSTGRES_DB}";
CREATE DATABASE "${POSTGRES_DB}";
SQL

echo "Restoring from $BACKUP..."
PGPASSWORD="$POSTGRES_PASSWORD" pg_restore \
  -h "$PGHOST" \
  -p "$PGPORT" \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  --no-owner \
  --no-privileges \
  "$BACKUP"

echo "✓ Restore complete"
