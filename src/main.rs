use std::{
    env,
    net::SocketAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::NaiveDateTime;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tower_http::cors::CorsLayer;

const ABSOLUTE_OUTPUT_PROMPT: &str = include_str!("../prompts/gpt-6-luna_absolute_prompt.txt");

#[derive(Clone)]
struct AppState {
    db: PgPool,
    http: reqwest::Client,
    jwt_secret: String,
    openai_key: String,
}

struct ApiError(StatusCode, &'static str);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

#[derive(Deserialize)]
struct Credentials {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    #[serde(rename = "userId")]
    user_id: i32,
    exp: u64,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    #[serde(rename = "userId")]
    user_id: i32,
}

#[derive(FromRow)]
struct User {
    id: i32,
    password: String,
}

#[derive(Deserialize)]
struct SummarizeRequest {
    article: String,
}

#[derive(Serialize)]
struct SummarizeResponse {
    summary: String,
}

#[derive(Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
}

#[derive(Deserialize)]
struct CompletionChoice {
    message: CompletionMessage,
}

#[derive(Deserialize)]
struct CompletionMessage {
    content: Option<String>,
}

#[derive(FromRow)]
struct HistoryRow {
    id: i32,
    original: String,
    summary: String,
    created_at: NaiveDateTime,
}

#[derive(Serialize)]
struct HistoryItem {
    id: i32,
    original: String,
    summary: String,
    created_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quickblog=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let jwt_secret = env::var("JWT_SECRET")?;
    let openai_key = env::var("OPENAI_API_KEY")?;
    if jwt_secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters".into());
    }

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    let state = AppState {
        db,
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .build()?,
        jwt_secret,
        openai_key,
    };
    let origin: HeaderValue = env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_owned())
        .parse()?;
    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/summarize", post(summarize).get(history))
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let address: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "QuickBlog API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn token_for(user_id: i32, secret: &str) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token"))?
        .as_secs();
    let claims = Claims {
        user_id,
        exp: now + 7 * 24 * 60 * 60,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| {
        tracing::error!(%error, "JWT signing failed");
        ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
    })
}

fn authenticated_user(headers: &HeaderMap, secret: &str) -> Result<i32, ApiError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(ApiError(StatusCode::UNAUTHORIZED, "Unauthorized"))?;
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ApiError(StatusCode::UNAUTHORIZED, "Invalid token"))?
    .claims;
    if claims.user_id <= 0 {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Invalid token"));
    }
    Ok(claims.user_id)
}

async fn signup(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> Result<Json<TokenResponse>, ApiError> {
    let email = input.email.trim().to_lowercase();
    if email.len() > 254
        || !email.contains('@')
        || input.password.len() < 8
        || input.password.len() > 72
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Enter a valid email and a password of 8 to 72 bytes",
        ));
    }

    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(input.password, 10))
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create account",
            )
        })?
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create account",
            )
        })?;
    let user_id: i32 =
        sqlx::query_scalar("INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
            .bind(email)
            .bind(hash)
            .fetch_one(&state.db)
            .await
            .map_err(|error| {
                if let sqlx::Error::Database(ref database_error) = error {
                    if database_error.code().as_deref() == Some("23505") {
                        return ApiError(StatusCode::CONFLICT, "User already exists");
                    }
                }
                tracing::error!(%error, "Signup database query failed");
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create account",
                )
            })?;
    Ok(Json(TokenResponse {
        token: token_for(user_id, &state.jwt_secret)?,
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = input.email.trim().to_lowercase();
    let user: Option<User> =
        sqlx::query_as("SELECT id, password FROM users WHERE lower(email) = $1")
            .bind(email)
            .fetch_optional(&state.db)
            .await
            .map_err(|error| {
                tracing::error!(%error, "Login database query failed");
                ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to sign in")
            })?;
    let user = user.ok_or(ApiError(StatusCode::UNAUTHORIZED, "Invalid credentials"))?;
    let verified =
        tokio::task::spawn_blocking(move || bcrypt::verify(input.password, &user.password))
            .await
            .map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to sign in"))?
            .map_err(|_| ApiError(StatusCode::UNAUTHORIZED, "Invalid credentials"))?;
    if !verified {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Invalid credentials"));
    }
    Ok(Json(AuthResponse {
        token: token_for(user.id, &state.jwt_secret)?,
        user_id: user.id,
    }))
}

async fn summarize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, ApiError> {
    let user_id = authenticated_user(&headers, &state.jwt_secret)?;
    let article = input.article.trim();
    if article.is_empty() || article.chars().count() > 20_000 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Article must contain 1 to 20,000 characters",
        ));
    }

    let response = state.http.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&state.openai_key)
        .json(&json!({
            "model": "gpt-6-luna",
            "messages": [
                { "role": "developer", "content": ABSOLUTE_OUTPUT_PROMPT },
                { "role": "developer", "content": "Summarize the article clearly and concisely. Keep its main facts and conclusions. Do not add facts or introductory filler." },
                { "role": "user", "content": article }
            ]
        }))
        .send().await
        .map_err(|error| {
            tracing::error!(%error, "OpenAI request failed");
            ApiError(StatusCode::BAD_GATEWAY, "Summarization service is unavailable")
        })?;
    if !response.status().is_success() {
        tracing::error!(status = %response.status(), "OpenAI rejected summarization request");
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "Summarization service is unavailable",
        ));
    }
    let completion: CompletionResponse = response.json().await.map_err(|error| {
        tracing::error!(%error, "OpenAI response could not be parsed");
        ApiError(
            StatusCode::BAD_GATEWAY,
            "Summarization service returned an invalid response",
        )
    })?;
    let summary = completion
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or(ApiError(
            StatusCode::BAD_GATEWAY,
            "Summarization service returned an empty summary",
        ))?;

    sqlx::query("INSERT INTO summaries (original, summary, user_id) VALUES ($1, $2, $3)")
        .bind(article)
        .bind(&summary)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Saving summary failed");
            ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save summary")
        })?;
    Ok(Json(SummarizeResponse { summary }))
}

async fn history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<HistoryItem>>, ApiError> {
    let user_id = authenticated_user(&headers, &state.jwt_secret)?;
    let rows: Vec<HistoryRow> = sqlx::query_as(
        "SELECT id, original, summary, created_at FROM summaries WHERE user_id = $1 ORDER BY created_at DESC, id DESC")
        .bind(user_id).fetch_all(&state.db).await
        .map_err(|error| {
            tracing::error!(%error, "History database query failed");
            ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch summaries")
        })?;
    Ok(Json(
        rows.into_iter()
            .map(|row| HistoryItem {
                id: row.id,
                original: row.original,
                summary: row.summary,
                created_at: format!("{}Z", row.created_at.format("%Y-%m-%dT%H:%M:%S%.3f")),
            })
            .collect(),
    ))
}
