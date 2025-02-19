use futures::TryFutureExt;
use tokio::fs::create_dir_all;

#[derive(Debug, thiserror::Error)]
pub enum IOError {
    #[error("{source}, path: {path}")]
    IOPathError {
        #[source]
        source: std::io::Error,
        path: String,
    },
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl IOError {
    pub fn from(source: std::io::Error) -> Self {
        Self::IOError(source)
    }
    pub fn with_path(source: std::io::Error, path: impl AsRef<std::path::Path>) -> Self {
        let path = path.as_ref();

        Self::IOPathError {
            source,
            path: path.to_string_lossy().to_string(),
        }
    }
}

pub async fn read_async(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, IOError> {
    let path_ref = path.as_ref();

    tokio::fs::read(path_ref)
        .await
        .map_err(|err| IOError::with_path(err, path_ref.to_string_lossy().to_string()))
}

pub async fn write_async(
    path: impl AsRef<std::path::Path>,
    data: impl AsRef<[u8]>,
) -> Result<(), IOError> {
    let path_ref = path.as_ref();
    let data_ref = data.as_ref();

    if path_ref.is_dir() {
        create_dir_all(path_ref).await?;
    } else {
        create_dir_all(path_ref.parent().unwrap()).await?;
    }

    tokio::fs::write(path_ref, data_ref)
        .await
        .map_err(|err| IOError::with_path(err, path_ref.to_string_lossy().to_string()))
}

pub async fn read_json_async<T>(path: impl AsRef<std::path::Path>) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    read_async(&path)
        .err_into::<crate::Error>()
        .await
        .and_then(|ref it| Ok(serde_json::from_slice(it)?))
}

pub async fn read_toml_async<T>(path: impl AsRef<std::path::Path>) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    read_async(&path)
        .err_into::<crate::Error>()
        .await
        .and_then(|ref it| {
            let toml_str = std::str::from_utf8(it).map_err(|_| {
                crate::ErrorKind::NoValueFor(format!("Can't read TOML at {:?}", path.as_ref()))
                    .as_error()
            })?;
            Ok(toml::from_str(toml_str)?)
        })
}

pub async fn write_json_async<T>(path: impl AsRef<std::path::Path>, data: T) -> crate::Result<()>
where
    T: serde::Serialize,
{
    Ok(write_async(path, serde_json::to_vec(&data)?).await?)
}

pub async fn write_toml_async<T>(path: impl AsRef<std::path::Path>, data: T) -> crate::Result<()>
where
    T: serde::Serialize,
{
    Ok(write_async(path, toml::to_string(&data)?).await?)
}

// dunce canonicalize
pub fn canonicalize(path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf, IOError> {
    let path = path.as_ref();
    dunce::canonicalize(path).map_err(|e| IOError::IOPathError {
        source: e,
        path: path.to_string_lossy().to_string(),
    })
}

pub async fn rename(
    from: impl AsRef<std::path::Path>,
    to: impl AsRef<std::path::Path>,
) -> Result<(), IOError> {
    let from = from.as_ref();
    let to = to.as_ref();
    tokio::fs::rename(from, to)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: from.to_string_lossy().to_string(),
        })
}

pub async fn remove_file(path: impl AsRef<std::path::Path>) -> Result<(), IOError> {
    let path = path.as_ref();
    tokio::fs::remove_file(path)
        .await
        .map_err(|e| IOError::IOPathError {
            source: e,
            path: path.to_string_lossy().to_string(),
        })
}
