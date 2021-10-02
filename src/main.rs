use backoff::future::retry;
use backoff::ExponentialBackoff;
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use reqwest::{header, Client};
use std::env;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static START_INSTANT: Lazy<Instant> = Lazy::new(Instant::now);

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&START_INSTANT);
    let client = Client::new();
    env::args()
        // Skip the filename itself
        .skip(1)
        .map(|url| {
            let client = &client;
            async move { retry(create_backoff(), || try_request_url(client, &url)).await }
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<()>()
        .await?;
    Ok(())
}

fn create_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_millis(5 * 60_000)),
        ..ExponentialBackoff::default()
    }
}

async fn try_request_url(client: &Client, url: &str) -> Result<(), backoff::Error<reqwest::Error>> {
    eprintln!("{} requesting {}...", timestamp(), url);
    client
        .get(url)
        .header(header::USER_AGENT, USER_AGENT)
        .send()
        .await?;
    eprintln!("{} got {}", timestamp(), url);
    Ok(())
}

fn timestamp() -> impl Display {
    struct DisplayDuration(Duration);
    impl Display for DisplayDuration {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{:5}.{:03}]", self.0.as_secs(), self.0.subsec_millis())
        }
    }
    DisplayDuration(Instant::now().duration_since(*START_INSTANT))
}
