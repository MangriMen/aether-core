use bytes::Bytes;

pub async fn sha1_async<T>(input: T) -> crate::Result<String>
where
    T: Into<Bytes> + Send,
{
    let bytes = input.into();
    Ok(tokio::task::spawn_blocking(move || sha1_smol::Sha1::from(bytes).hexdigest()).await?)
}
