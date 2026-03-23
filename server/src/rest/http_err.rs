use axum::http;
use serde_json::json;

pub fn bad_request(msg: String) -> axum::response::Response {
    let msg_json = json!({"status": 400, "msg": msg.as_str()}).to_string();

    return axum::response::Response::builder()
        .status(http::StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(msg_json))
        .unwrap(); // TODO: #yolo
}
