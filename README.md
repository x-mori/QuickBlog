# QuickBlog

Paste an article, get a concise summary, and find it later in your account history.

The frontend is React and Vite in `client/`. The API is Rust with Axum, Tokio, SQLx, PostgreSQL, and Reqwest. The API calls OpenAI's `gpt-4o-mini` model.

## Run locally

You need Rust and Cargo, Node.js, PostgreSQL, and an OpenAI API key. Create a PostgreSQL database, then copy `.env.example` to `.env` in the repository root and set `DATABASE_URL`, `JWT_SECRET`, and `OPENAI_API_KEY`. Use a random `JWT_SECRET` of at least 32 characters.

```bash
npm install
npm install --prefix client
npm run dev
```

`npm run dev` starts `cargo run` and Vite. The API listens on `127.0.0.1:3000` and Vite proxies `/api` to it. You can also run them separately with `cargo run` and `npm run client`. On the first API start, SQLx applies the files in `migrations/`.

For a separately hosted frontend, set `VITE_API_URL` to the API origin when building the client. Set `CORS_ORIGIN` on the API to the frontend origin. See `client/.env.example`. Build the frontend with `npm run build --prefix client`.

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
