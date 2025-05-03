use std::{sync::Arc, time};

use lazy_static::lazy_static;
use reqwest_retry::policies::ExponentialBackoff;
use tokio::sync::Semaphore;

const FETCH_ATTEMPTS: u32 = 5;

lazy_static! {
    pub static ref REQWEST_CLIENT: Arc<reqwest_middleware::ClientWithMiddleware> = {
        let client = reqwest::Client::builder()
            .tcp_keepalive(Some(time::Duration::from_secs(10)))
            .build()
            .expect("Failed to build reqwest client");

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(FETCH_ATTEMPTS);
        let retry_middleware =
            reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy);

        let client_with_middlewares = reqwest_middleware::ClientBuilder::new(client)
            .with(retry_middleware)
            .build();

        Arc::new(client_with_middlewares)
    };
}

#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);
