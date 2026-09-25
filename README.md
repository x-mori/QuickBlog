# QuickBlog

Paste an article, get a concise summary, and find it later in your account history.

QuickBlog is a Rust workspace. The Axum API lives in `src/`; the Yew WebAssembly client lives in `client/`. PostgreSQL stores accounts and summaries. The API uses OpenAI's `gpt-4o-mini` model.

## Run locally

Install Rust and Cargo, PostgreSQL, and Trunk. Add the browser target and install Trunk once:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Create a PostgreSQL database. Copy `.env.example` to `.env` at the repository root and set `DATABASE_URL`, `JWT_SECRET`, and `OPENAI_API_KEY`. Use a random `JWT_SECRET` of at least 32 characters.

In one terminal, run the API:

```bash
cargo run
```

In another terminal, run the client:

```bash
cd client
trunk serve
```

Open `http://127.0.0.1:8080`. Trunk proxies `/api` to the API on `127.0.0.1:3000`. SQLx applies `migrations/` when the API starts.

To build the client for separate hosting, run `trunk build --release` inside `client/`. Set `QUICKBLOG_API_URL` to the API origin before building if the frontend and API use different origins, and set `CORS_ORIGIN` on the API to the frontend origin. Configure the static host to return `index.html` for client routes such as `/history` and `/login`.

## API

| Method | Route | Result |
| --- | --- | --- |
| `POST` | `/api/auth/signup` | Create an account and return `{ "token" }` |
| `POST` | `/api/auth/login` | Sign in and return `{ "token", "userId" }` |
| `POST` | `/api/summarize` | Summarize `{ "article" }`, save it, and return `{ "summary" }` |
| `GET` | `/api/summarize` | Return the signed-in user's `{ "id", "summary", "created_at" }` rows |

The summary routes require `Authorization: Bearer <token>`. New tokens expire after seven days. Paste 1 to 20,000 characters per article. API errors have an `error` field and a matching HTTP status.

## Existing databases

The first migration creates the missing `users` table and summary ownership column when needed. It keeps existing account IDs and bcrypt password hashes. Old summary rows without an owner remain in the database but cannot appear in any user's history. Existing tokens from the Express API have no expiration claim, so users must sign in again after switching to Rust. Back up a production database before applying the migration.
