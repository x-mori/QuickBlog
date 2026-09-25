use base64::Engine as _;
use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{HtmlInputElement, HtmlTextAreaElement, InputEvent, SubmitEvent};
use yew::prelude::*;
use yew_router::prelude::*;

type Auth = UseStateHandle<Option<String>>;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signup")]
    Signup,
    #[at("/history")]
    History,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Serialize)]
struct Credentials {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Serialize)]
struct ArticleRequest {
    article: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    summary: String,
}

#[derive(Deserialize)]
struct HistoryItem {
    id: i32,
    summary: String,
    created_at: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

fn api_url(path: &str) -> String {
    let base = option_env!("QUICKBLOG_API_URL")
        .unwrap_or("")
        .trim_end_matches('/');
    format!("{base}{path}")
}

fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn token_is_current(token: &str) -> bool {
    let Some(payload) = token.split('.').nth(1) else {
        return false;
    };
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload) else {
        return false;
    };
    let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    claims
        .get("exp")
        .and_then(|exp| exp.as_u64())
        .is_some_and(|exp| exp as f64 > js_sys::Date::now() / 1000.0)
}

fn stored_token() -> Option<String> {
    let storage = browser_storage()?;
    let token = storage.get_item("token").ok().flatten()?;
    if token_is_current(&token) {
        Some(token)
    } else {
        let _ = storage.remove_item("token");
        None
    }
}

fn save_token(auth: &Auth, token: Option<String>) {
    if let Some(storage) = browser_storage() {
        match &token {
            Some(value) => {
                let _ = storage.set_item("token", value);
            }
            None => {
                let _ = storage.remove_item("token");
            }
        }
    }
    auth.set(token);
}

#[hook]
fn use_auth() -> Auth {
    use_context::<Auth>().expect("auth provider must wrap the app")
}

async fn read_response<T: DeserializeOwned>(response: Response) -> Result<T, String> {
    if response.ok() {
        response
            .json()
            .await
            .map_err(|_| "The server returned an invalid response".to_owned())
    } else {
        let status = response.status();
        let body: Result<ErrorResponse, _> = response.json().await;
        Err(body
            .map(|body| body.error)
            .unwrap_or_else(|_| format!("Request failed ({status})")))
    }
}

#[component]
fn App() -> Html {
    let auth = use_state(stored_token);
    let year = js_sys::Date::new_0().get_full_year();
    html! {
        <ContextProvider<Auth> context={auth}>
            <BrowserRouter>
                <div class="site">
                    <Header />
                    <main class="site-main">
                        <Switch<Route> render={switch} />
                    </main>
                    <footer class="footer">{format!("© {year} made by fujimori_")}</footer>
                </div>
            </BrowserRouter>
        </ContextProvider<Auth>>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::Login => html! { <AuthPage signup={false} /> },
        Route::Signup => html! { <AuthPage signup={true} /> },
        Route::History => html! { <HistoryPage /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home} /> },
    }
}

#[component]
fn Header() -> Html {
    let auth = use_auth();
    let navigator = use_navigator().expect("router must wrap header");
    if auth.is_none() {
        return Html::default();
    }
    let logout = {
        let auth = auth.clone();
        Callback::from(move |_| {
            save_token(&auth, None);
            navigator.push(&Route::Login);
        })
    };
    html! {
        <nav class="nav" aria-label="Main navigation">
            <Link<Route> to={Route::Home}>{"Generator"}</Link<Route>>
            <Link<Route> to={Route::History}>{"History"}</Link<Route>>
            <button type="button" onclick={logout}>{"Logout"}</button>
        </nav>
    }
}

#[derive(Properties, PartialEq)]
struct AuthPageProps {
    signup: bool,
}

#[component]
fn AuthPage(props: &AuthPageProps) -> Html {
    let auth = use_auth();
    let navigator = use_navigator().expect("router must wrap auth page");
    let email = use_state(String::new);
    let password = use_state(String::new);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let signup = props.signup;

    let on_email = {
        let email = email.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            email.set(input.value());
        })
    };
    let on_password = {
        let password = password.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            password.set(input.value());
        })
    };
    let onsubmit = {
        let auth = auth.clone();
        let navigator = navigator.clone();
        let email = email.clone();
        let password = password.clone();
        let error = error.clone();
        let loading = loading.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if *loading {
                return;
            }
            loading.set(true);
            error.set(None);
            let auth = auth.clone();
            let navigator = navigator.clone();
            let error = error.clone();
            let loading = loading.clone();
            let credentials = Credentials {
                email: (*email).clone(),
                password: (*password).clone(),
            };
            spawn_local(async move {
                let path = if signup {
                    "/api/auth/signup"
                } else {
                    "/api/auth/login"
                };
                let result = async {
                    let request = Request::post(&api_url(path))
                        .json(&credentials)
                        .map_err(|_| "Could not prepare the request".to_owned())?;
                    let response = request
                        .send()
                        .await
                        .map_err(|_| "Could not reach the API".to_owned())?;
                    read_response::<TokenResponse>(response).await
                }
                .await;
                match result {
                    Ok(result) if token_is_current(&result.token) => {
                        save_token(&auth, Some(result.token));
                        navigator.push(&Route::Home);
                    }
                    Ok(_) => error.set(Some("The server returned an invalid token".to_owned())),
                    Err(message) => error.set(Some(message)),
                }
                loading.set(false);
            });
        })
    };
    let (heading, action, alternate, alternate_route, alternate_action) = if signup {
        (
            "Sign up an account",
            "Sign up",
            "Have an account?",
            Route::Login,
            "Login",
        )
    } else {
        (
            "Login to your account",
            "Login",
            "Don't have an account?",
            Route::Signup,
            "Sign up",
        )
    };
    html! {
        <section class="auth">
            <h1>{heading}</h1>
            <form {onsubmit}>
                <label for="email">{"Enter your email:"}</label>
                <input id="email" type="email" value={(*email).clone()} oninput={on_email} required=true autocomplete="email" />
                <label for="password">{"Enter your password:"}</label>
                <input id="password" type="password" value={(*password).clone()} oninput={on_password} required=true autocomplete={if signup {"new-password"} else {"current-password"}} />
                <button class="primary" type="submit" disabled={*loading}>
                    {if *loading {"Working..."} else {action}}
                </button>
            </form>
            {if let Some(message) = &*error { html! { <p class="error" role="alert">{message}</p> } } else { Html::default() }}
            <p class="auth-link">{alternate}{" "}<Link<Route> to={alternate_route}>{alternate_action}</Link<Route>></p>
        </section>
    }
}

#[component]
fn HomePage() -> Html {
    let auth = use_auth();
    let article = use_state(String::new);
    let summary = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let copied = use_state(|| false);

    let on_article = {
        let article = article.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target_unchecked_into();
            article.set(input.value());
        })
    };
    let onsubmit = {
        let auth = auth.clone();
        let article = article.clone();
        let summary = summary.clone();
        let error = error.clone();
        let loading = loading.clone();
        let copied = copied.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if *loading {
                return;
            }
            let Some(token) = (*auth).clone() else {
                return;
            };
            error.set(None);
            summary.set(None);
            copied.set(false);
            loading.set(true);
            let auth = auth.clone();
            let summary = summary.clone();
            let error = error.clone();
            let loading = loading.clone();
            let article = (*article).clone();
            spawn_local(async move {
                let result = async {
                    let request = Request::post(&api_url("/api/summarize"))
                        .header("Authorization", &format!("Bearer {token}"))
                        .json(&ArticleRequest { article })
                        .map_err(|_| "Could not prepare the request".to_owned())?;
                    let response = request
                        .send()
                        .await
                        .map_err(|_| "Could not reach the API".to_owned())?;
                    if response.status() == 401 {
                        save_token(&auth, None);
                    }
                    read_response::<SummaryResponse>(response).await
                }
                .await;
                match result {
                    Ok(result) => summary.set(Some(result.summary)),
                    Err(message) => error.set(Some(message)),
                }
                loading.set(false);
            });
        })
    };
    let on_copy = {
        let summary = summary.clone();
        let copied = copied.clone();
        Callback::from(move |_| {
            let Some(text) = (*summary).clone() else {
                return;
            };
            let Some(window) = web_sys::window() else {
                return;
            };
            let promise = window.navigator().clipboard().write_text(&text);
            let copied = copied.clone();
            spawn_local(async move {
                if JsFuture::from(promise).await.is_ok() {
                    copied.set(true);
                }
            });
        })
    };
    if auth.is_none() {
        return html! { <Redirect<Route> to={Route::Login} /> };
    }
    html! {
        <section>
            <h1>{"QuickBlog"}</h1>
            <div class="workspace">
                <div>
                    <h2>{"Prompt"}</h2>
                    <form class="editor" {onsubmit}>
                        <textarea rows="6" value={(*article).clone()} oninput={on_article} placeholder="Paste your blog/article here..." required=true />
                        <button class="primary" type="submit" disabled={*loading}>
                            {if *loading {"Summarizing..."} else {"Summarize"}}
                        </button>
                    </form>
                    {if let Some(message) = &*error { html! { <p class="error" role="alert">{message}</p> } } else { Html::default() }}
                </div>
                <div>
                    <h2>{"Summary"}</h2>
                    <div class="result" aria-live="polite">
                        {if *loading { html! { "Loading..." } }
                         else if let Some(text) = &*summary { html! { text } }
                         else { html! { <span class="placeholder">{"Your summary appears here..."}</span> } }}
                    </div>
                    <button class="copy" type="button" onclick={on_copy} disabled={summary.is_none()}>
                        {if *copied {"Copied!"} else {"Copy summary"}}
                    </button>
                </div>
            </div>
        </section>
    }
}

#[component]
fn HistoryPage() -> Html {
    let auth = use_auth();
    let items = use_state(Vec::<HistoryItem>::new);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let token = (*auth).clone();
    {
        let auth = auth.clone();
        let items = items.clone();
        let error = error.clone();
        let loading = loading.clone();
        use_effect_with(token.clone(), move |token| {
            if let Some(token) = token.clone() {
                let auth = auth.clone();
                let items = items.clone();
                let error = error.clone();
                let loading = loading.clone();
                spawn_local(async move {
                    let result = async {
                        let response = Request::get(&api_url("/api/summarize"))
                            .header("Authorization", &format!("Bearer {token}"))
                            .send()
                            .await
                            .map_err(|_| "Could not reach the API".to_owned())?;
                        if response.status() == 401 {
                            save_token(&auth, None);
                        }
                        read_response::<Vec<HistoryItem>>(response).await
                    }
                    .await;
                    match result {
                        Ok(history) => items.set(history),
                        Err(message) => error.set(Some(message)),
                    }
                    loading.set(false);
                });
            }
            || ()
        });
    }
    if token.is_none() {
        return html! { <Redirect<Route> to={Route::Login} /> };
    }
    html! {
        <section class="history">
            <h1>{"Summary History"}</h1>
            {if *loading { html! { <p>{"Loading..."}</p> } } else { Html::default() }}
            {if let Some(message) = &*error { html! { <p class="error" role="alert">{message}</p> } } else { Html::default() }}
            {if !*loading && error.is_none() && items.is_empty() {
                html! { <p class="empty">{"No summaries yet."}</p> }
            } else { Html::default() }}
            {for items.iter().map(|item| html! {
                <article key={item.id}>
                    <time>{item.created_at.replace('T', " ").trim_end_matches('Z')}{" UTC"}</time>
                    <p>{&item.summary}</p>
                </article>
            })}
        </section>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
