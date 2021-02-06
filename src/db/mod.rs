mod manager;
mod messages;
use sqlx::{Postgres, postgres::PgPoolOptions, Pool};
pub use manager::DBManager;
pub use messages::DBMessage;

pub async fn connect<'a>(db_url: &'a str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url).await?;
    Ok(pool)
}
