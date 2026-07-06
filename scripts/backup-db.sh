#!/usr/bin/env bash
# Backup PostgreSQL database.
# Usage: ./scripts/backup-db.sh [output-path]
#
# Defaults: ./backups/backup-YYYY-MM-DD-HHMMSS.dump

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

OUTPUT="${1:-backups/backup-$(date +%Y-%m-%d-%H%M%S).dump}"
mkdir -p "$(dirname "$OUTPUT")"

# Load env
if [ -f .env.prod ]; then
  set -a
  . .env.prod
  set +a
fi

: "${POSTGRES_USER:?POSTGRES_USER not set}"
: "${POSTGRES_PASSWORD:?POSTGRES_PASSWORD not set}"
: "${POSTGRES_DB:=foodcity}"

# Detect host — default to localhost, override with PGHOST if needed
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"

echo "Backing up ${POSTGRES_DB} at ${PGHOST}:${PGPORT} → ${OUTPUT}"
PGPASSWORD="$POSTGRES_PASSWORD" pg_dump \
  -Fc \
  -h "$PGHOST" \
  -p "$PGPORT" \
  -U "$POSTGRES_USER" \
  "$POSTGRES_DB" \
  > "$OUTPUT"

SIZE=$(du -h "$OUTPUT" | cut -f1)
echo "✓ Backup complete: $OUTPUT ($SIZE)"

# Retention: keep last 14 days
find "$(dirname "$OUTPUT")" -name "backup-*.dump" -mtime +14 -delete || true
echo "✓ Old backups pruned (kept last 14 days)"
