# QuickBlog client

Run `npm install --prefix client`, then `npm run client` from the repository root. Vite proxies `/api` to the Rust API on `127.0.0.1:3000` during development.

When hosting the frontend separately, set `VITE_API_URL` to the Rust API origin before running `npm run build --prefix client`. The API must allow that frontend origin through `CORS_ORIGIN`.
