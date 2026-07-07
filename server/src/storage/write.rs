use crate::models;
use crate::storage::storage;
use sqlx::{FromRow, QueryBuilder, Sqlite};

const INSERT_SONG: &str = r#"
    INSERT INTO song (id, title, duration_ms, artist_id, album_id, track_number)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
"#;

const INSERT_ARTIST: &str = r#"
    INSERT INTO artist (id, name)
    VALUES (?1, ?2)
"#;

const INSERT_ALBUM: &str = r#"
    INSERT INTO album (id, artist_id, name, release_date, track_count, duration_ms)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
"#;

const INSERT_SONG_TOKEN: &str = r#"
    INSERT INTO song_token (song_id, token)
    VALUES (?1, ?2)
"#;

const INSERT_ARTIST_TOKEN: &str = r#"
    INSERT INTO artist_token (artist_id, token)
    VALUES (?1, ?2)
"#;

const INSERT_ALBUM_TOKEN: &str = r#"
    INSERT INTO album_token (album_id, token)
    VALUES (?1, ?2)
"#;

const INSERT_PLAYBACK_LOG: &str = r#"
    INSERT INTO playback_log (id, start, end, duration_ms, song_id, artist_id, album_id, skipped)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#;

impl storage::Storage {
    pub async fn write_full_song(&self) -> Result<(), sqlx::Error> {
        // begin transaction

        // call many other methods

        // commit

        return Ok(());
    }

    fn insert_song(&self, tx: sqlx::sqlite::SqliteTransaction) -> Result<(), sqlx::Error> {
        // use tx
        // Fetch

        // Check for any new data to write

        // Update if necessary

        // commit
        return Ok(());
    }

    fn upsert_artist(&self, tx: sqlx::sqlite::SqliteTransaction) -> Result<(), sqlx::Error> {
        // use tx
        // Fetch

        // Check for any new data to write

        // Update if necessary

        // commit
        return Ok(());
    }

    fn upsert_album(&self, tx: sqlx::sqlite::SqliteTransaction) -> Result<(), sqlx::Error> {
        // use tx
        // Fetch

        // Check for any new data to write

        // Update if necessary

        // commit

        return Ok(());
    }
}
