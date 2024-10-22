use bytes::Bytes;

pub async fn sha1_async(bytes: Bytes) -> crate::Result<String> {
    let hash =
        tokio::task::spawn_blocking(move || sha1_smol::Sha1::from(bytes).hexdigest()).await?;

    Ok(hash)
}
