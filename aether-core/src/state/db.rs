use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

pub(crate) async fn connect(database_path: &Path) -> crate::Result<Pool<Sqlite>> {
    let uri = format!("sqlite:{}", database_path.display());

    if !Sqlite::database_exists(&uri).await? {
        Sqlite::create_database(&uri).await?;
    }

    let conn_options = SqliteConnectOptions::from_str(&uri)?
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None);

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect_with(conn_options)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
