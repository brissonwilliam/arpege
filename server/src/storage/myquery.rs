pub static CREATE_DB_QUERIES: &[&'static str] = &[
    r#"
        CREATE TABLE IF NOT EXISTS song (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            name VARCHAR(256) NOT NULL,
            duration INTEGER NOT NULL,
            artist_id BLOB(16),
            album_id BLOB(16),
            track_number INT8
        )
    "#,
    // https://www.sqlite.org/fts5.html
    "CREATE VIRTUAL TABLE IF NOT EXISTS search_idx USING fts5 (song_name, album_name, artist_name)",
    "CREATE INDEX IF NOT EXISTS idx_song_created ON song(created DESC)",
    "CREATE INDEX IF NOT EXISTS idx_song_album ON song(album_id, track_number)",
    "CREATE INDEX IF NOT EXISTS idx_duration ON song(duration)",
    r#"
        CREATE TABLE IF NOT EXISTS artist (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            name VARCHAR(128) NOT NULL
        )
    "#,
    r#"
        CREATE TABLE IF NOT EXISTS album (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            duration INTEGER NOT NULL,
            name VARCHAR(128) NOT NULL
        );
    "#,
    r#"
        CREATE TABLE IF NOT EXISTS user (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            name VARCHAR(128) NOT NULL
        );
    "#,
];
