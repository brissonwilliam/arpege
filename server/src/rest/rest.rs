use axum::{body, http, response, routing, Router};
use log;

pub async fn start() {
    let host = "0.0.0.0:8222";
    log::info!("starting web api on {host}");

    let app = Router::new().route("/", routing::get(hello));

    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn get_song_md() {
    // query db: does it exist?
    // Return allll the good info!
    // Plus audio file info as well
}

// Not sure how this is going to be called
/*
async fn get_song_chunk() -> response::Response<body::Bytes> {
    // TODO: read identifier from http requset
    // Query db to get fs path and update metrics
    // TODO: figure out playlists?

    let path = "./data/ato.m4a";
    let my_data: Vec<u8> = streamer::read_chunk(path, 0).unwrap();
    let body_bytes = body::Bytes::from(my_data);

    let resp = response::Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "application/octet-stream") // Or other appropriate content type
        .body(body::Bytes::from(body_bytes))
        .unwrap(); // Handle potential errors in a real application

    return resp;
}
    */

/*
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}
*/
