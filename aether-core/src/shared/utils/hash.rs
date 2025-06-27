use bytes::Bytes;
use tokio::task::{spawn_blocking, JoinError};

pub async fn sha1_async<T>(input: T) -> Result<String, JoinError>
where
    T: Into<Bytes> + Send,
{
    let bytes = input.into();
    spawn_blocking(move || sha1_smol::Sha1::from(bytes).hexdigest()).await
}
