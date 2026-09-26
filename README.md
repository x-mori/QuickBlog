# QuickBlog

Paste an article, get a concise summary, and find it later in your account history.

QuickBlog is a Rust workspace. The Axum API lives in `src/`; the Yew WebAssembly client lives in `client/`. PostgreSQL stores accounts and summaries. The API uses OpenAI's `gpt-6-luna` model and sends `prompts/gpt-6-luna_absolute_prompt.txt` as a developer message for summaries.

## Run locally

Install Node.js 22 or newer, Rust and Cargo, and PostgreSQL. Add the browser target once:

```bash
rustup target add wasm32-unknown-unknown
```

The repository has a local `.env` with a generated `JWT_SECRET`. Put your key in `OPENAI_API_KEY`, then run:

```bash
npm run dev
```

Open `http://127.0.0.1:8080`. On first run, the command installs Trunk in `.local/tools`, initializes PostgreSQL in `.local/postgres`, and starts the API and frontend. Press Ctrl+C to stop them. The local database and summary history remain for the next run. `DATABASE_URL` can stay empty; the command supplies a local connection URL. To use an existing PostgreSQL database, set `DATABASE_URL` in `.env` instead. Restart `npm run dev` after changing `.env`.

To build the client for separate hosting, use the Trunk executable in `.local/tools/bin` to run `trunk build --release` inside `client/`. Set `QUICKBLOG_API_URL` to the API origin before building if the frontend and API use different origins, and set `CORS_ORIGIN` on the API to the frontend origin. Configure the static host to return `index.html` for client routes such as `/history` and `/login`.

## API

| Method | Route | Result |
| --- | --- | --- |
| `POST` | `/api/auth/signup` | Create an account and return `{ "token" }` |
| `POST` | `/api/auth/login` | Sign in and return `{ "token", "userId" }` |
| `POST` | `/api/summarize` | Summarize `{ "article" }`, save it, and return `{ "summary" }` |
| `GET` | `/api/summarize` | Return the signed-in user's `{ "id", "original", "summary", "created_at" }` rows |

The summary routes require `Authorization: Bearer <token>`. New tokens expire after seven days. Paste 1 to 20,000 characters per article. API errors have an `error` field and a matching HTTP status.

## Existing databases

The first migration creates the missing `users` table and summary ownership column when needed. It keeps existing account IDs and bcrypt password hashes. Old summary rows without an owner remain in the database but cannot appear in any user's history. Existing tokens from the Express API have no expiration claim, so users must sign in again after switching to Rust. Back up a production database before applying the migration.
