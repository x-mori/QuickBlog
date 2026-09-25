# QuickBlog client

This directory contains the Yew WebAssembly frontend. Install its build tools with `rustup target add wasm32-unknown-unknown` and `cargo install --locked trunk`.

Run `npm run dev` from the repository root to start the local database, API, and frontend together. For a separate setup, run `trunk serve` here while `cargo run` serves the API from the repository root. Trunk proxies `/api` to the local API. Run `trunk build --release` here for a production client bundle in `dist/`.

For a separately hosted API, set `QUICKBLOG_API_URL` to its origin before building and set the API's `CORS_ORIGIN` to the frontend origin. Serve `index.html` for browser routes such as `/history`.
