# Food City — Operations Runbook

> Practical guide for operating Food City in production.
> Covers deployment, incident response, common ops tasks.

---

## 1. Service Architecture

```
[Customer Web] ─┐
[Restaurant] ───┼──► [Nginx/CDN] ──► [Axum API] ──► [PostgreSQL primary]
[Driver App] ───┤         │              │              ▲
[Admin Console] ┘         │              ├──► [Redis]   ├─ [Read replica]
                         WS             ───► [Stripe]
```

### Services

| Service | Port | Purpose |
|---|---|---|
| backend | 8080 | Axum REST + WS API |
| frontend | 80 (nginx) | Static React build |
| postgres | 5432 | Primary DB |
| redis | 6379 | Cache + pub/sub + GEO |

### Health checks

- `GET /health` — liveness (always returns `ok`)
- `GET /ready` — readiness (pings DB + Redis)
- `GET /metrics` — Prometheus metrics

---

## 2. Deployment

### Production deploy (Docker Compose)

```bash
# 1. Pull latest
git pull origin main

# 2. Build images
docker compose -f docker-compose.prod.yml build

# 3. Backup DB
./scripts/backup-db.sh

# 4. Deploy (zero-downtime if using watchtower or rolling K8s)
docker compose -f docker-compose.prod.yml up -d

# 5. Run migrations (if not auto-run on startup)
docker compose exec backend sqlx migrate run

# 6. Verify
curl https://api.foodcity.app/ready
```

### Rolling back

```bash
# Tag-based: deploy previous image
docker compose -f docker-compose.prod.yml up -d --no-deps backend:previous-tag

# DB rollback (if migration is reversible):
sqlx migrate revert
```

---

## 3. Database

### Backup

```bash
# Daily snapshot
pg_dump -Fc -U foodcity -h localhost foodcity > backup-$(date +%F).dump

# Restore
pg_restore -U foodcity -h localhost -d foodcity < backup-YYYY-MM-DD.dump
```

### Migration

Migrations auto-run on backend startup. For zero-downtime:

1. **Add nullable column** (safe — old code ignores it)
2. **Backfill in batches** (`UPDATE ... WHERE id BETWEEN ... LIMIT 1000`)
3. **Add NOT NULL constraint** (after backfill)
4. **Deploy new code** that uses the column

Dangerous operations requiring maintenance window:
- `DROP COLUMN` (use `ALTER ... RENAME` + drop after deploy)
- `ALTER COLUMN ... TYPE`
- Long-running `CREATE INDEX` (use `CREATE INDEX CONCURRENTLY`)

### Read replica setup

```env
DATABASE_REPLICA_URL=postgres://foodcity:...@replica-host:5432/foodcity
```

Routes marked read-only use `state.db.read()` (replica), writes go to `state.db` (primary).

---

## 4. Redis

### Cache invalidation

```bash
# Invalidate restaurant list cache for a city
redis-cli --scan --pattern 'restaurants:list:*' | xargs redis-cli DEL

# Invalidate a single restaurant's menu cache
redis-cli DEL menu:restaurant:UUID

# Flush all rate limit counters (emergency)
redis-cli --scan --pattern 'rl:*' | xargs redis-cli DEL
```

### Driver location set

```bash
# See all drivers currently in the GEO set
redis-cli ZRANGE drivers:locations 0 -1 WITHSCORES

# Find drivers near a point (lat,lng,radius)
redis-cli GEORADIUS drivers:locations 79.86 6.92 5000 m ASC WITHDIST
```

---

## 5. Incidents

### "Order stuck in pending_accept"

Likely cause: restaurant dashboard not loaded, or restaurant offline.

```sql
-- Check order age
SELECT id, placed_at, EXTRACT(EPOCH FROM NOW() - placed_at) AS age_seconds
FROM orders WHERE status = 'pending_accept' ORDER BY placed_at;
```

Actions:
- Auto-reject orders older than 5 min (worker handles this; verify it's running)
- Ping restaurant owner via admin console

### "No drivers accepting orders"

```bash
# Check available drivers
redis-cli SMEMBERS drivers:available

# Check heartbeats
redis-cli KEYS 'driver:hb:*' | xargs -L1 redis-cli TTL
```

Actions:
- If no available drivers: surge pricing, notify drivers via push
- If heartbeats expired: drivers' apps crashed; restart them

### "Payment succeeded but order stuck"

Check `payment_intents` vs `orders.payment_status`:

```sql
SELECT o.id, o.status, o.payment_status, pi.status as intent_status
FROM orders o
LEFT JOIN payment_intents pi ON pi.order_id = o.id
WHERE o.status = 'pending_accept' AND o.payment_status = 'pending'
  AND pi.status = 'succeeded';
```

This means webhook raced with sync response but state machine guard failed. Manual fix:

```sql
UPDATE orders SET payment_status = 'succeeded' WHERE id = '...';
```

### "Stripe webhook not arriving"

1. Check Stripe dashboard → Webhooks → recent events for our endpoint
2. Verify `STRIPE_WEBHOOK_SECRET` matches the signing secret
3. Check our `payment_webhooks` table for received events:

```sql
SELECT * FROM payment_webhooks ORDER BY created_at DESC LIMIT 10;
```

4. If events arriving but not processed, replay from Stripe dashboard

### "WebSocket connections dropping"

1. Check `/metrics` for `ws_connections_active`
2. Check Redis pub/sub is alive: `redis-cli PUBLISH test "hello"`
3. Check backend memory — WS connections are stateful, OOM kills them
4. Scale out: add more backend replicas behind LB (sticky sessions not required)

### "DB connection pool exhausted"

Symptom: API returns 503 with "database error" or hangs.

```bash
# Active connections
psql -c "SELECT state, COUNT(*) FROM pg_stat_activity GROUP BY state;"
```

Actions:
- Increase `DATABASE_MAX_CONNECTIONS` env var (restarts backend)
- Add read replica for read-heavy endpoints
- Check for slow queries: `pg_stat_statements`

---

## 6. On-call checklist

When paged:

1. **Acknowledge** within 5 min
2. **Check `/ready`** — is the service up at all?
3. **Check `/metrics`** — any obvious anomalies (high `db_pool_size`, low `db_pool_idle`)?
4. **Check recent deploys** — was there a migration? Code change?
5. **Tail logs**: `docker compose logs -f backend | jq .`
6. **Notify** in #incidents channel
7. **Resolve or escalate**

---

## 7. Performance

### Hot endpoints (cache strategy)

| Endpoint | Cache | TTL | Invalidation |
|---|---|---|---|
| `GET /restaurants` | Redis | 30s | Time-based |
| `GET /restaurants/:id` | Redis | 60s | On restaurant update |
| `GET /restaurants/:id/menu` | Redis | 60s | On menu publish |
| `GET /restaurants/cuisines` | Redis | 5min | Time-based |

### Slow query log

```sql
ALTER SYSTEM SET log_min_duration_statement = 500; -- log queries > 500ms
SELECT pg_reload_conf();
```

### Load testing

```bash
# Install k6
brew install k6

# Test order placement flow
k6 run scripts/loadtest-order-placement.js
```

---

## 8. Security

### Rotate JWT secret (forces all users to re-login)

```bash
# 1. Generate new secret
openssl rand -base64 48

# 2. Update env
echo "JWT_SECRET=<new>" >> backend/.env

# 3. Restart backend
docker compose restart backend
```

### Stripe key rotation

1. Generate new key in Stripe dashboard
2. Update `STRIPE_SECRET_KEY` env var
3. Restart backend
4. Old key keeps working until you disable it in Stripe (graceful)

### Suspicious activity

- Brute force on `/auth/login`: rate limit kicks in (5/min/IP). Confirm via Redis:
  ```bash
  redis-cli GET rl:ip:1.2.3.4:login
  ```
- Promo abuse: check `promo_redemptions` for clusters from same IP/device
- Account takeover: check `users` for recent password changes + unusual addresses

---

## 9. Cost monitoring

### Daily cost report

```sql
SELECT
    DATE(placed_at) AS day,
    COUNT(*) AS orders,
    SUM(total_cents) / 100.0 AS gmv_usd,
    SUM(delivery_fee_cents) / 100.0 AS delivery_revenue_usd,
    SUM(tip_cents) / 100.0 AS tips_usd
FROM orders
WHERE status = 'delivered' AND placed_at > NOW() - INTERVAL '30 days'
GROUP BY day ORDER BY day DESC;
```

### Payout reconciliation

```sql
SELECT
    o.id,
    o.total_cents,
    pl.payee_type,
    pl.amount_cents,
    pl.status,
    pl.stripe_transfer_id
FROM orders o
JOIN payout_ledger pl ON pl.order_id = o.id
WHERE o.delivered_at > NOW() - INTERVAL '7 days'
  AND pl.status != 'paid'
ORDER BY o.delivered_at;
```

Unpaid payouts → check Stripe dashboard for failed transfers.

---

## 10. Disaster recovery

### Full DB loss

1. Provision new PG instance
2. Restore from latest backup: `pg_restore -d foodcity < backup.dump`
3. Update `DATABASE_URL`
4. Restart backend
5. Verify counts: `SELECT COUNT(*) FROM orders;`

### Redis data loss (cache miss storm)

Acceptable — cache will rebuild from DB. Watch for thundering herd:
- `singleflight` lock in `cache_service.rs` prevents it
- If still degraded, temporarily lower `DATABASE_MAX_CONNECTIONS` to protect DB

### Stripe outage

- Webhooks queue at Stripe end (3-day retry)
- `payment_reconciler` worker polls Stripe API every 5 min as backup
- No customer impact for new orders (mock mode in dev; in prod, fail gracefully)

---

## 11. Useful SQL snippets

### Active orders by restaurant

```sql
SELECT r.name, COUNT(*) AS active_orders
FROM orders o
JOIN restaurants r ON r.id = o.restaurant_id
WHERE o.status IN ('pending_accept','accepted','preparing','ready','picked_up','delivering')
GROUP BY r.name
ORDER BY active_orders DESC;
```

### Top drivers this week

```sql
SELECT d.id, COUNT(o.id) AS deliveries, SUM(o.tip_cents)/100.0 AS tips_usd
FROM drivers d
JOIN orders o ON o.driver_id = d.id
WHERE o.delivered_at > NOW() - INTERVAL '7 days'
GROUP BY d.id
ORDER BY deliveries DESC
LIMIT 20;
```

### Low-rated restaurants (need attention)

```sql
SELECT name, rating_avg, rating_count
FROM restaurants
WHERE rating_count > 10 AND rating_avg < 3.5
ORDER BY rating_avg ASC;
```
