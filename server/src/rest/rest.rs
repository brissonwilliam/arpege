use log;
use axum::{routing, Router };

pub async fn start() {
    log::info!("starting web api");

    let app = Router::new().route("/", routing::get(hello));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8222").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn hello() -> &'static str {
    "Hello, World!"
}

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
