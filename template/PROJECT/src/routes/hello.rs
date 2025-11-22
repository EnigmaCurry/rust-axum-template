use axum::{extract::Path, routing::get, Router};

pub fn router() -> Router {
    // this router is responsible for everything under `/hello`
    Router::new()
        .route("/{name}", get(hello))
        .route("/", get(hello_default))
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}

async fn hello_default() -> &'static str {
    "Hello!"
}
