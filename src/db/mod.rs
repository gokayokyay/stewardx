mod manager;
mod messages;
pub use manager::DBManager;
pub use messages::DBMessage;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, migrate};
use anyhow::Result;

pub async fn connect<'a>(db_url: &'a str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    let migrator = migrate!();
    match migrator.run(&pool).await {
        Ok(_) => {},
        Err(e) => {
            panic!("Couldn't migrate!, {}", e.to_string());
        }
    };
    Ok(pool)
}
