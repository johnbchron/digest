#![feature(iterator_try_collect)]

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, LazyLock},
};

use chrono::Duration;
pub use feed_rs;
use feed_rs::model::{Entry, Feed};
use miette::{Context, IntoDiagnostic, Result};
use reqwest::{
  Client, Url,
  header::{ACCEPT_ENCODING, HeaderValue, USER_AGENT},
};
use tokio::{io::AsyncReadExt, task::JoinSet};
use tracing::{error, info, instrument};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Clone)]
pub struct FeedState {
  feeds: Arc<HashMap<String, Feed>>,
}

impl FeedState {
  pub async fn new() -> Result<FeedState> {
    Ok(FeedState {
      feeds: Arc::new(fetch_all_feeds().await?),
    })
  }

  pub fn last_hours(&self, hours: usize) -> Vec<(Feed, Vec<Entry>)> {
    let mut results = Vec::new();
    for feed in self.feeds.values() {
      let mut entries = feed
        .entries
        .iter()
        .filter(|e| {
          e.updated.is_some_and(|u| {
            u > chrono::Utc::now() - Duration::hours(hours as _)
          })
        })
        .cloned()
        .collect::<Vec<_>>();
      entries.sort_unstable_by_key(|e| e.updated.unwrap());

      results.push((feed.clone(), entries));
    }

    results.retain(|(_, e)| !e.is_empty());

    results.sort_unstable_by_key(|(f, _)| f.id.clone());
    results
  }
}

#[instrument]
async fn read_feed_url_list() -> Result<Vec<Url>> {
  let feed_list_file_path = std::env::var("FEED_LIST_FILE")
    .into_diagnostic()
    .context("missing `FEED_LIST_FILE` env var")?;
  let feed_list_file_path = PathBuf::from(&feed_list_file_path);

  let mut feed_list_file = tokio::fs::File::open(&feed_list_file_path)
    .await
    .into_diagnostic()
    .context(format!(
      "failed to open feed list file: {feed_list_file_path:?}"
    ))?;
  let mut feed_list_string = String::new();
  feed_list_file
    .read_to_string(&mut feed_list_string)
    .await
    .into_diagnostic()
    .context("failed to read feed list file")?;

  feed_list_string
    .lines()
    .map(|l| Url::parse(l.trim()))
    .try_collect()
    .into_diagnostic()
    .context("failed to parse URL from feed list")
}

#[instrument(skip_all, fields(url = url.to_string()))]
async fn fetch_feed_from_url(url: Url) -> Result<Option<Feed>> {
  info!("fetching feed");

  let req = HTTP_CLIENT
    .get(url.clone())
    .header(
      USER_AGENT,
      HeaderValue::from_static(
        "User-Agent: Digest/0.1.0 (+https://digest.jlewis.sh; \
         contact@jlewis.sh)",
      ),
    )
    .header(
      ACCEPT_ENCODING,
      HeaderValue::from_static("brotli, zstd, gzip, deflate"),
    );
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context(format!("failed to fetch feed for url \"{url}\""))?;
  let resp = match resp.error_for_status() {
    Ok(resp) => resp,
    Err(e) => {
      if let Some(status) = e.status() {
        error!(
          "got {num} ({desc}) error from origin",
          num = status.as_u16(),
          desc = status.canonical_reason().unwrap_or_default()
        );
        return Ok(None);
      } else {
        unreachable!("got non-status error from `error_for_status`")
      }
    }
  };
  let body = resp
    .text()
    .await
    .into_diagnostic()
    .context(format!("failed to read response body for feed: \"{url}\""))?;

  let feed = feed_rs::parser::parse(std::io::Cursor::new(&body))
    .into_diagnostic()
    .context(format!("failed to parse feed: \"{url}\": {body:?}"))?;
  info!("successfully fetched feed");

  Ok(Some(feed))
}

#[instrument]
async fn fetch_all_feeds() -> Result<HashMap<String, Feed>> {
  let urls = read_feed_url_list()
    .await
    .context("failed to read feed url list")?;

  let mut js = JoinSet::new();
  for url in urls {
    js.spawn(fetch_feed_from_url(url));
  }

  let mut map = HashMap::new();
  while let Some(res) = js.join_next().await {
    let feed = res
      .into_diagnostic()
      .context("failed to join feed fetch task")?
      .context("failed to fetch feed from url")?;
    let Some(feed) = feed else { continue };
    map.insert(feed.id.clone(), feed);
  }

  Ok(map)
}
