use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::Router,
  };  
use rust_embed::Embed;
use mime_guess;

static INDEX_HTML: &str = "index.html";

#[derive(Embed)]
#[folder = "src/svelte/build/"]
struct Assets;

#[tokio::main]
async fn main() {
    let app = Router::new().fallback(static_handler);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
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
                (header::CACHE_CONTROL, "public, max-age=86400, immutable"),
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
    (StatusCode::NOT_FOUND, "404").into_response()
  }