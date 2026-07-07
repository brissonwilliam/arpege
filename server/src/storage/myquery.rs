pub static CREATE_DB_QUERIES: &[&'static str] = &[
    r#"
        CREATE TABLE IF NOT EXISTS song (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            title VARCHAR(1024) NOT NULL,
            duration_ms INTEGER NOT NULL,
            artist_id BLOB(16),
            album_id BLOB(16),
            track_number INT8
        )
    "#,
    // https://www.sqlite.org/fts5.html
    // "CREATE VIRTUAL TABLE IF NOT EXISTS search_idx USING fts5 (song_name, album_name, artist_name)",
    "CREATE INDEX IF NOT EXISTS idx_song_created ON song(created DESC)",
    "CREATE INDEX IF NOT EXISTS idx_song_album ON song(album_id, track_number)",
    "CREATE INDEX IF NOT EXISTS idx_duration ON song(duration_ms)",
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
            artist_id BLOB(16) NOT NULL,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            name VARCHAR(128) NOT NULL,
            release_date DATETIME NOT NULL,
            track_count INT8 NOT NULL,
            duration_ms INTEGER NOT NULL
        );
    "#,
    /* #TOKEN #token #SEARCH #search #METADATA #metadata */
    r#"
        CREATE TABLE IF NOT EXISTS song_token (
            song_id BLOB(16) PRIMARY KEY,
            token CHAR(2) NOT NULL
        )
    "#,
    r#"
        CREATE TABLE IF NOT EXISTS artist_token (
            artist_id BLOB(16) PRIMARY KEY,
            token CHAR(2) NOT NULL
        )
    "#,
    r#"
        CREATE TABLE IF NOT EXISTS album_token (
            album_id BLOB(16) PRIMARY KEY,
            token CHAR(2) NOT NULL
        )
    "#,
    "CREATE INDEX IF NOT EXISTS idx_song_tokens ON song_token (token)",
    "CREATE INDEX IF NOT EXISTS idx_artist_tokens ON artist_token (token)",
    /* PLAYBACK playback meta META */
    // Denomarlize some data for better OLAP
    r#"
        CREATE TABLE IF NOT EXISTS playback_log (
            id BLOB(16) PRIMARY KEY,
            start DATETIME NOT NULL,
            end DATETIME NOT NULL,
            duration_ms INTEGER NOT NULL,
            song_id BLOB(16) NOT NULL,
            artist_id BLOB(16),
            album_id BLOB(16),
            skipped BOOLEAN
        )
    "#,
    /* #USERS #users */
    "CREATE INDEX IF NOT EXISTS idx_album_token ON album_token (token)",
    r#"
        CREATE TABLE IF NOT EXISTS user (
            id BLOB(16) PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            name VARCHAR(128) NOT NULL
        );
    "#,
];
