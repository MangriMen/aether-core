use std::future::Future;

use futures::TryStream;

use crate::features::events::LoadingBarId;

use super::emit_loading;

// A drop in replacement to try_for_each_concurrent that emits loading events as it goes
// Key is the key to use for which loading bar- a LoadingBarId. If None, does nothing
// Total is the total amount of progress that the loading bar should take up by all futures in this (will be split evenly amongst them).
// If message is Some(t) you will overwrite this loading bar's message with a custom one
// futures_count is the number of futures that will be run, which is needed as we allow Iterator to be passed in, which doesn't have a size
#[tracing::instrument(skip(stream, f))]

pub async fn loading_try_for_each_concurrent<I, F, Fut, T>(
    stream: I,
    limit: Option<usize>,
    key: Option<&LoadingBarId>,
    total: f64,
    futures_count: usize, // num is in here as we allow Iterator to be passed in, which doesn't have a size
    message: Option<&str>,
    f: F,
) -> crate::Result<()>
where
    I: futures::TryStreamExt<Error = crate::Error> + TryStream<Ok = T>,
    F: FnMut(T) -> Fut + Send,
    Fut: Future<Output = crate::Result<()>> + Send,
    T: Send,
{
    let mut f = f;
    stream
        .try_for_each_concurrent(limit, |item| {
            let f = f(item);
            async move {
                f.await?;
                if let Some(key) = key {
                    emit_loading(key, total / (futures_count as f64), message).await?;
                }
                Ok(())
            }
        })
        .await
}
