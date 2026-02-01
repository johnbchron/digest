use std::pin::Pin;

use axum::{
  body::Body,
  extract::State,
  http::{Request, Response},
};
use leptos::{IntoView, prelude::provide_context};

use crate::AppState;

#[allow(clippy::type_complexity)]
pub(crate) fn handle_page<Vf, Iv>(
  app_fn: Vf,
) -> impl Fn(
  State<AppState>,
  Request<Body>,
) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'static>>
+ Clone
+ Send
+ Sync
+ 'static
where
  Vf: Fn() -> Iv + Clone + Send + Sync + 'static,
  Iv: IntoView + 'static,
{
  move |State(app_state): State<AppState>, request: Request<Body>| {
    leptos_axum::render_app_to_stream_with_context(
      context_provider(app_state.clone()),
      {
        let app_fn = app_fn.clone();
        move || {
          site_app::shell(app_state.stylesheet_content.clone(), app_fn.clone())
        }
      },
    )(request)
  }
}

#[allow(clippy::type_complexity)]
pub(crate) fn handle_fragment<Vf, Iv>(
  app_fn: Vf,
) -> impl Fn(
  State<AppState>,
  Request<Body>,
) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'static>>
+ Clone
+ Send
+ Sync
+ 'static
where
  Vf: Fn() -> Iv + Clone + Send + Sync + 'static,
  Iv: IntoView + 'static,
{
  move |State(app_state): State<AppState>, request: Request<Body>| {
    leptos_axum::render_app_to_stream_with_context(
      context_provider(app_state.clone()),
      app_fn.clone(),
    )(request)
  }
}

fn context_provider(app_state: AppState) -> impl Fn() + Clone {
  move || {
    provide_context(app_state.feeds.clone());
  }
}
