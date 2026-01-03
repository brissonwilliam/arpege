use super::myquery;
use log;
use sqlx;
use std;
use std::fs::OpenOptions;

pub async fn get_pool() -> Result<sqlx::Pool<sqlx::Sqlite>, Box<dyn std::error::Error>> {
    log::info!("initializing storage");
    const DB_FILE: &str = "data/aperge.db";
    OpenOptions::new().write(true).create(true).open(DB_FILE)?;

    log::info!("connecting to database file");
    let mut connect_str = String::from("sqlite://");
    connect_str.push_str(DB_FILE);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(connect_str.as_str())
        .await?;
    ensure_ready(&pool).await?;

    return Ok(pool);
}

// creates the database if it does not exist
async fn ensure_ready(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    for db_query in myquery::CREATE_DB_QUERIES {
        log::debug!("exeucting: {db_query}");
        sqlx::query(db_query)
            .execute(pool)
            .await
            .inspect_err(|err| {
                log::error!("could not execute query. error: {err}. \nQuery:\n{db_query}")
            })?;
    }
    return Ok(());
}

#[derive(Debug, Clone)]
pub struct Storage {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl Storage {
    pub fn clone(&self) -> Storage {
        return Storage {
            pool: self.pool.clone(),
        };
    }
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Storage {
        return Storage { pool };
    }
}
