use axum::{
  http::{header, StatusCode, Uri},
  response::{Html, IntoResponse, Response},
  routing::Router,
};
use rust_embed::RustEmbed;
use mime_guess;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use dotenvy::dotenv;
use std::env;
use anyhow::Result;
use tokio::time::Duration;
use tower_http::{
    services::ServeDir,
    compression::CompressionLayer,
    set_header::SetResponseHeaderLayer
};
use axum::http::{HeaderName, HeaderValue};

static INDEX_HTML: &str = "index.html";

#[derive(RustEmbed)]
#[folder = "src/svelte/build/"]
struct Assets;

#[tokio::main]
async fn main() -> Result<()> {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL")?;
  println!("Using database URL: {}", database_url);

  let pool = SqlitePoolOptions::new()
    .max_connections(10)
    .min_connections(5)
    .max_lifetime(Some(Duration::from_secs(60 * 60)))
    .idle_timeout(Some(Duration::from_secs(30)))
    .test_before_acquire(true)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&database_url).await?;

  create_todos_table(&pool).await?;

  // let description = "hello";

  // let id = add_todo(&pool, description).await?;
  // println!("Added todo with id: {}", id);

  list_todos(&pool).await?;

  let app = Router::new()
    .nest_service("/assets", ServeDir::new("assets"))
    .fallback(static_handler)
    .layer(CompressionLayer::new().br(true))
    .layer(SetResponseHeaderLayer::overriding(
      HeaderName::from_static("cache-control"),
      HeaderValue::from_static("public, max-age=86400, immutable"),
  ));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
      .await
      .unwrap();
  println!("Listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();

  Ok(())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
  let path = uri.path().trim_start_matches('/');

  if path.is_empty() || path == INDEX_HTML {
      return index_html().await;
  }

  match Assets::get(path) {
      Some(content) => {
          let mime = mime_guess::from_path(path).first_or_octet_stream();
          (
              [
                  (header::CONTENT_TYPE, mime.as_ref()),
                  // (header::CACHE_CONTROL, "public, max-age=86400, immutable"),
              ],
              content.data,
          )
              .into_response()
      }
      None => {
          if path.contains('.') {
              return not_found().await;
          }
          index_html().await
      }
  }
}

async fn index_html() -> Response {
  match Assets::get(INDEX_HTML) {
      Some(content) => Html(content.data).into_response(),
      None => not_found().await,
  }
}

async fn not_found() -> Response {
  (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

async fn add_todo(pool: &SqlitePool, description: &str) -> Result<i64> {
  let mut conn = pool.acquire().await?;

  let id = sqlx::query!(
      r#"
      INSERT INTO todos (description)
      VALUES (?1)
      "#,
      description
  )
  .execute(&mut *conn)
  .await?
  .last_insert_rowid();

  Ok(id)
}

async fn list_todos(pool: &SqlitePool) -> Result<()> {
  let recs = sqlx::query!(
      r#"
      SELECT id, description, done
      FROM todos
      ORDER BY id
      "#
  )
  .fetch_all(pool)
  .await?;

  for rec in recs {
      println!(
          "- [{}] {}: {}",
          if rec.done { "x" } else { " " },
          rec.id,
          &rec.description,
      );
  }

  Ok(())
}

async fn create_todos_table(pool: &SqlitePool) -> Result<()> {
  let mut conn = pool.acquire().await?;
  
  sqlx::query!(
      r#"
      CREATE TABLE IF NOT EXISTS todos
(
    id          INTEGER PRIMARY KEY NOT NULL,
    description TEXT                NOT NULL,
    done        BOOLEAN             NOT NULL DEFAULT 0
)
      "#
  )
  .execute(&mut *conn)
  .await?;

  println!("Tabel 'todos' berhasil dibuat atau sudah ada.");

  Ok(())
}
