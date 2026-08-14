# Microgreens Store

A small full-stack storefront for selling microgreens, built on top of the
salability logic in `examples/microgreens/salability.py`. Standalone example,
unrelated to Headroom's core compression library.

- `backend/` — REST API in Rust (Axum + SQLx/SQLite + JWT auth)
- `frontend/` — Angular storefront (catalog, cart, checkout, order history, admin)

## Features

- **Catalog** — lists only batches that are currently salable (within their
  variety's harvest window and passing quality grading).
- **Cart & checkout** — add items, apply a discount code (`FRESH10` is
  seeded by default), place an order. Stock is decremented on checkout.
- **Accounts** — register/login with JWT auth; orders are tied to the
  logged-in customer.
- **Admin** — add new batches and record quality checks (mold, color
  uniformity, stem/root condition, height), reusing the same grading rules
  as the CLI tool.

## Running locally

### Backend

```bash
cd backend
cargo run
# listens on http://localhost:8080, creates ./microgreens.db on first run
```

The first registered user is a regular customer. Promote one to admin
manually, e.g.:

```bash
sqlite3 backend/microgreens.db "UPDATE users SET role='admin' WHERE email='you@example.com';"
```

### Frontend

```bash
cd frontend
npm install
npm start   # ng serve, http://localhost:4200
```

The frontend talks to the backend at `http://localhost:8080/api`
(see `frontend/src/environments/environment.ts`).

## Testing

### Backend

```bash
cd backend
cargo test
# 10 unit tests for the salability grading rules (models.rs) +
# 9 integration tests exercising the full HTTP API (tests/api.rs) against
# an in-memory SQLite database.
```

### Frontend

```bash
cd frontend
npm install
CHROME_BIN=/path/to/chromium npx ng test --watch=false --browsers=ChromeHeadlessCI
# 19 unit tests covering AuthService, CartService, ProductService, the auth
# HTTP interceptor, and the auth/admin route guards.
```

`karma.conf.js` defines the `ChromeHeadlessCI` launcher (adds `--no-sandbox`,
needed when running as root in a container). Omit `--browsers` to run
interactively against a local Chrome/Chromium instead.

## API overview

| Method | Path                          | Auth  | Description                        |
|--------|-------------------------------|-------|-------------------------------------|
| POST   | `/api/auth/register`          | -     | Create an account                   |
| POST   | `/api/auth/login`             | -     | Log in, returns a JWT               |
| GET    | `/api/products`                | -     | List salable batches                |
| GET    | `/api/varieties`               | -     | List known microgreen varieties     |
| GET    | `/api/cart`                    | user  | View cart                           |
| POST   | `/api/cart/items`              | user  | Add/update a cart line              |
| DELETE | `/api/cart/items/:id`          | user  | Remove a cart line                  |
| POST   | `/api/checkout`                | user  | Place an order (optional discount)  |
| GET    | `/api/orders`                  | user  | Order history                       |
| POST   | `/api/admin/batches`           | admin | Register a new grow batch           |
| POST   | `/api/admin/batches/:id/inspect` | admin | Record a quality check          |
