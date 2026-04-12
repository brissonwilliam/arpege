use axum::body::Body;
use serde::Serialize;
use serde_json;

#[derive(Serialize)]
struct ChunkMeta {
    id: String,
    start_ms: u32,
    end_ms: u32,
    duration_ms: u32,
}

#[derive(Serialize)]
struct SongMd {
    id: String,
    title: String,
    artist: String,
    duration_ms: u32,
    mime_type: String, // e.g. "audio/mpeg", "audio/mp4; codecs=\"mp4a.40.2\"", "audio/ogg; codecs=opus"
    chunks: Vec<ChunkMeta>,
}

pub async fn get_song_md() -> axum::response::Response {
    // TODO:
    // read query param song id
    // db check for existance
    // read md from db

    // TODO: much later, add caching for md

    let song = SongMd {
        id: "123-456".to_owned(),
        title: "24".to_owned(),
        artist: "ato".to_owned(),
        duration_ms: 243438,
        mime_type: "audio/mp4".to_owned(),
        chunks: vec![
            ChunkMeta {
                id: "out_000.m4a".to_owned(),
                start_ms: 0,
                end_ms: 10007,
                duration_ms: 10007,
            },
            ChunkMeta {
                id: "out_001.m4a".to_owned(),
                start_ms: 1007,
                end_ms: 10007 + 10007,
                duration_ms: 10007,
            },
            ChunkMeta {
                id: "out_002.m4a".to_owned(),
                start_ms: 20016,
                end_ms: 20016 + 9984,
                duration_ms: 9984,
            },
            ChunkMeta {
                id: "out_003.m4a".to_owned(),
                start_ms: 30000,
                end_ms: 30000 + 10007,
                duration_ms: 10007,
            },
        ],
    };
    let song_json = serde_json::to_vec(&song).unwrap(); // TODO: #yolo

    let resp = axum::http::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "application/json") // Or other appropriate content type
        .body(Body::from(song_json))
        .unwrap(); // TODO: #yolo

    return resp;
}
