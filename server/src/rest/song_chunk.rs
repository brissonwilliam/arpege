use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Query;

use crate::rest::http_err;

pub async fn get_song_chunk(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
    // TODO:
    // read query param song id
    // query db for path

    let chunk_id = params.get("c"); // TODO: #yolo
    if chunk_id.is_none() {
        return http_err::bad_request("invalid 'c' query param".to_owned());
    }

    let dat = read_chunk(chunk_id.unwrap()).unwrap(); // TODO: #yolo. If not found, log internal
                                                      // err, return internal err

    let resp = axum::http::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(Body::from(dat))
        .unwrap(); // TODO: #yolo

    return resp;
}

fn read_chunk(id: &str) -> std::io::Result<Vec<u8>> {
    let path: String = "./data/fs/transcodes/".to_owned() + id;
    return std::fs::read(path);
}
