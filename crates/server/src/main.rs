mod handle;

use std::{io::Read, path::PathBuf, sync::Arc};

use axum::{
  Router,
  body::Body,
  extract::{FromRef, State},
  http::Request,
  routing::get,
};
use feeds::FeedState;
use miette::{Context, IntoDiagnostic};
use site_app::HomePage;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, prelude::*};

use self::handle::handle_page;

#[derive(Clone, FromRef)]
struct AppState {
  stylesheet_content: Arc<str>,
  static_asset_dir:   PathBuf,
  feeds:              FeedState,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
  let env_filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy();
  tracing_subscriber::registry()
    .with(env_filter)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let stylesheet_path = std::env::var("STYLESHEET_PATH")
    .into_diagnostic()
    .context("failed to read env var `STYLESHEET_PATH`")?;
  let mut stylesheet_content = String::new();
  match std::fs::File::open(&stylesheet_path) {
    Ok(mut f) => {
      f.read_to_string(&mut stylesheet_content)
        .into_diagnostic()
        .context("failed to read from stylesheet file")?;
    }
    Err(e) => {
      tracing::warn!("failed to open stylesheet file: {e}");
    }
  };
  let stylesheet_content = Arc::<str>::from(stylesheet_content);

  let static_asset_dir = PathBuf::from(
    std::env::var("STATIC_ASSET_DIR")
      .into_diagnostic()
      .context("failed to read env var `STATIC_ASSET_DIR`")?,
  );

  let feed_state = FeedState::new()
    .await
    .context("failed to build feed state")?;

  let app_state = AppState {
    stylesheet_content,
    static_asset_dir,
    feeds: feed_state,
  };

  let _ = any_spawner::Executor::init_tokio();
  let router = Router::new()
    .route("/", get(handle_page(HomePage)))
    .fallback(|State(app_state): State<AppState>, req: Request<Body>| {
      ServeDir::new(&app_state.static_asset_dir).oneshot(req)
    })
    .with_state(app_state);

  let listener = tokio::net::TcpListener::bind("[::]:3000").await.unwrap();

  info!(addr = ?listener.local_addr().unwrap(), "listening to socket");
  axum::serve(listener, router).await.unwrap();

  Ok(())
}
