# QuickBlog

Paste a blog post, get a short summary, and find it again in your history.

> **Status:** The app currently has a React frontend and an Express API. A Rust API is planned. The Rust architecture below is the intended design, not an implemented service.

## What QuickBlog will do

- Summarize pasted article text with the OpenAI API.
- Save summaries to PostgreSQL for each signed-in user.
- Browse previous summaries in the history page.

The current UI accepts pasted text. The Express prompt formats that text today; a real summarization prompt is part of the Rust migration. Fetching articles from URLs is a possible later addition.

## Planned Rust architecture

```text
React + Vite frontend
        |
        | JSON API
        v
Rust API: Axum + Tokio
   |              |
   v              v
PostgreSQL      OpenAI API
   SQLx           Reqwest
```

The React app will stay in `client/`. The Rust API will replace the Express code in `server/` while keeping these routes and JSON responses:

| Method | Route | Purpose |
| --- | --- | --- |
| `POST` | `/api/auth/signup` | Create an account and return a token |
| `POST` | `/api/auth/login` | Sign in and return a token |
| `POST` | `/api/summarize` | Summarize article text and save the result |
| `GET` | `/api/summarize` | List the signed-in user's summaries |

The planned API will use Serde for JSON. It will verify existing bcrypt password hashes so current accounts can move to Rust.

## Repository layout

```text
client/   React and Vite frontend
server/   Current Express API, to be replaced by Rust
db/       Current SQL schema
```

## Development with the current API

You need Node.js and PostgreSQL. Clone the repository and install the dependencies:

```bash
git clone https://github.com/x-mori/QuickBlog.git
cd QuickBlog
```

Then install the current app dependencies:

```bash
npm install
npm install --prefix client
npm install --prefix server
```

Create `server/.env` for the current Express API:

```dotenv
DATABASE_URL=postgres://user:password@localhost:5432/quickblog
JWT_SECRET=replace-with-a-long-random-secret
OPENAI_API_KEY=your-api-key
PORT=3000
```

Run `npm run dev`. Express listens on port 3000 by default; Vite serves the frontend on its own development port.

The checked-in `db/schema.sql` is incomplete for a fresh database: the current API also needs a `users` table and a `summaries.user_id` column. Database migrations are part of the Rust work below.

## Rust migration

1. Add PostgreSQL migrations for accounts, summary ownership, and timestamps.
2. Implement the existing auth, summary, and history routes in Rust.
3. Change the model prompt to request an actual summary. Validate article size and report OpenAI or database failures clearly.
4. Make the frontend API URL configurable, verify the complete user flow, then retire the Express API.

Rust build and run instructions will be added when the Rust service exists.
