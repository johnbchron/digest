use std::sync::Arc;

use feeds::{
  FeedState,
  feed_rs::model::{Entry, Feed},
};
use leptos::prelude::*;

pub fn shell<F: Fn() -> Iv, Iv: IntoView>(
  stylesheet_content: Arc<str>,
  f: F,
) -> impl IntoView {
  view! {
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />

        <style>{ stylesheet_content }</style>

        <script src="/dist/htmx.min.js"></script>

        <title text="John's Daily Digest" />
      </head>
      <body class="min-h-dvh decoration-2">
        <main class="container mx-auto px-4 py-8">
          { f() }
        </main>
      </body>
    </html>
  }
}

#[component]
pub fn HomePage() -> impl IntoView {
  let feed_state = expect_context::<FeedState>();

  let last_24 = feed_state.last_hours(5 * 24);

  view! {
    <div class="flex flex-col gap-4">
      <p class="text-2xl">
        "john's daily digest"
      </p>

      <div class="h-[1px] bg-black self-stretch" />

      <div class="flex flex-col gap-4">
        { last_24.into_iter().map(|(f, e)| view! {
          <div>
            <FeedPostsByDay feed=&f entries=&e />
          </div>
        }).collect_view() }
      </div>
    </div>
  }
}

#[component]
fn FeedIcon<'a>(feed: &'a Feed) -> AnyView {
  let image = feed.icon.as_ref().or(feed.logo.as_ref());

  view! {
    <a
      href={ image.and_then(|i| i.link.clone()).map(|l| l.href) }
      class="size-5 rounded-sm border border-slate-600 inline"
    >
      { image.map(|i| view! { <img src=i.uri.clone() /> }) }
    </a>
  }
  .into_any()
}

#[component]
fn FeedPostsByDay<'a>(feed: &'a Feed, entries: &'a [Entry]) -> AnyView {
  view! {
    <ul>
      <li>
        // <div class="flex flex-row gap-2 items-center">
        //   <FeedIcon feed=feed />
          <p class="font-medium">
            { feed.title.as_ref().map(|t| t.content.clone()) }
          </p>
        // </div>
      </li>
      <li class="flex flex-col">
        <ul>
          { entries.iter().map(|e| view! {
            <li>
              <a
                href={ e.links.first().map(|l| l.href.clone()) }
                class="underline visited:text-gray-400 hover:no-underline"
                target="_blank" rel="noopener noreferrer"
              >
                { e.title.as_ref().map(|t| t.content.clone()) }
              </a>
            </li>
          }).collect_view() }
        </ul>
      </li>
    </ul>
  }
  .into_any()
}
