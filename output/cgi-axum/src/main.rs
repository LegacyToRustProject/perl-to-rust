// Converted from: test-projects/cgi-sample/hello.pl
// Original: Perl CGI.pm-based web application
//
// Conversion map:
//   my $q = CGI->new                         → Axum extractors (Query, Form, Json)
//   $q->param('name')                        → Query<HashMap> or Form<T> extraction
//   $q->header('text/html')                  → Html<String> return type
//   $q->header(-type => 'application/json')  → Json<T> return type
//   $q->header(-status => '404 ...')         → StatusCode + response tuple
//   $q->redirect($url)                       → Redirect::to(url)
//   DBI->connect / prepare / execute / fetch → sqlx::query_as / fetch_all
//   if ($action eq '...') { ... }            → axum::Router with route handlers
//   print $q->header(...); print $body       → return response (no global state)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::collections::HashMap;

// ── Shared application state ──────────────────────────────────────────────
// Perl: globals ($dbh, %config) → Rust: Arc<AppState> via Axum State extractor

#[derive(Clone)]
struct AppState {
    db: MySqlPool,
}

// ── Data models ───────────────────────────────────────────────────────────
// Perl: fetchrow_hashref() → anonymous hash → Rust: typed struct with FromRow

#[derive(Debug, Serialize, sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: Option<String>,
}

// ── Request parameter structs ─────────────────────────────────────────────
// Perl: $q->param('name') → Rust: Query<GreetQuery> or Form<GreetForm>

#[derive(Deserialize)]
struct GreetQuery {
    // Perl: $q->param('name') || 'World'
    #[serde(default = "default_world")]
    name: String,
}

fn default_world() -> String {
    "World".to_string()
}

#[derive(Deserialize)]
struct GreetForm {
    // Perl: $q->param('name') || 'Anonymous'
    #[serde(default = "default_anon")]
    name: String,
}

fn default_anon() -> String {
    "Anonymous".to_string()
}

// ── Endpoint 1: GET /  (Hello page) ──────────────────────────────────────
// Perl: $action eq 'index' → print $q->header('text/html'); print "<h1>Hello, $name!</h1>"
async fn index(Query(params): Query<GreetQuery>) -> Html<String> {
    let name = &params.name;
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Hello</title></head>
<body>
  <h1>Hello, {name}!</h1>
  <form method="post" action="/greet">
    Name: <input name="name" value="{name}">
    <button type="submit">Greet</button>
  </form>
</body>
</html>"#,
        name = name
    ))
}

// ── Endpoint 2: POST /greet  (form handler) ───────────────────────────────
// Perl: $action eq 'greet' → print $q->header(-charset => 'UTF-8'); print "<h2>Greetings, $name!</h2>"
async fn greet(Form(form): Form<GreetForm>) -> Html<String> {
    let name = &form.name;
    Html(format!(
        r#"<h2>Greetings, {name}!</h2><a href="/">Back</a>"#,
        name = name
    ))
}

// ── Endpoint 3: GET /users  (JSON API list) ───────────────────────────────
// Perl: $sth->execute(); while ($row = $sth->fetchrow_hashref()) { push @users, $row }
//       print $q->header('application/json');
async fn list_users(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let users: Vec<User> =
        sqlx::query_as("SELECT id, name, email FROM users ORDER BY id LIMIT 20")
            .fetch_all(&state.db)
            .await?;

    Ok(Json(serde_json::json!({ "users": users })))
}

// ── Endpoint 4: GET /users/:id  (single user lookup) ─────────────────────
// Perl: my $id = $q->param('id'); if ($id =~ /^\d+$/) { ... }
//       $dbh->selectrow_hashref("SELECT ... WHERE id = ?", undef, $id)
//       or: print $q->header(-status => '404 Not Found')
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user: Option<User> =
        sqlx::query_as("SELECT id, name, email FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    match user {
        Some(u) => Ok(Json(u)),
        // Perl: print $q->header(-status => '404 Not Found'); print "User not found"
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

// ── Endpoint 5: GET /redirect  ────────────────────────────────────────────
// Perl: print $q->redirect('?action=index')
async fn redirect_to_index() -> Redirect {
    Redirect::to("/")
}

// ── Error handling ────────────────────────────────────────────────────────
// Perl: print $q->header(-status => '400 Bad Request')
// Rust: custom error type that implements IntoResponse

enum AppError {
    Database(sqlx::Error),
    NotFound(String),
    BadRequest(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response(),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}

// ── Health check ──────────────────────────────────────────────────────────
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ── Router (replaces CGI action dispatch) ────────────────────────────────
// Perl: if ($action eq 'index') { ... } elsif ($action eq 'users') { ... }
// Rust: Router with typed routes — compile-time checked!
fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/greet", post(greet))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/redirect", get(redirect_to_index))
        .route("/health", get(health))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/mydb".to_string());

    let db = MySqlPool::connect(&database_url).await?;

    let state = AppState { db };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        // Tests without DB use a mock state
        // Real DB tests would use sqlx::test with test DB
        // Here we test routes that don't need the DB (index, greet, redirect, health)
        Router::new()
            .route("/", get(index))
            .route("/greet", post(greet))
            .route("/redirect", get(redirect_to_index))
            .route("/health", get(health))
    }

    #[tokio::test]
    async fn test_index_default_name() {
        // Perl: my $name = $q->param('name') || 'World';
        let app = test_router();
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(std::str::from_utf8(&body).unwrap().contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_index_custom_name() {
        // Perl: $q->param('name') → "Alice"
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/?name=Alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(std::str::from_utf8(&body).unwrap().contains("Hello, Alice!"));
    }

    #[tokio::test]
    async fn test_greet_post() {
        // Perl: $action eq 'greet' → POST handler
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/greet")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("name=Bob"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(std::str::from_utf8(&body).unwrap().contains("Greetings, Bob!"));
    }

    #[tokio::test]
    async fn test_greet_post_empty_name() {
        // Perl: $q->param('name') || 'Anonymous'
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/greet")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(""))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("Greetings, Anonymous!")
        );
    }

    #[tokio::test]
    async fn test_redirect() {
        // Perl: print $q->redirect('?action=index')
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/redirect")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // 3xx redirect
        assert!(resp.status().is_redirection());
        assert_eq!(resp.headers().get("location").unwrap(), "/");
    }

    #[tokio::test]
    async fn test_health() {
        let app = test_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("\"status\":\"ok\"")
        );
    }
}
