//! Poem target: full benchmark matrix (A–G, K–N).

use poem::{
    get, handler,
    listener::TcpListener,
    post,
    web::{Json, Path, Query},
    Endpoint, EndpointExt, IntoResponse, Middleware, Request, Response, Result, Route,
    Server,
};
use std::collections::HashMap;
use std::time::Instant;

/// Timing plus request-id middleware, enabled with MW=1.
struct BenchMiddleware;

impl<E: Endpoint> Middleware<E> for BenchMiddleware {
    type Output = BenchEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        BenchEndpoint(ep)
    }
}

struct BenchEndpoint<E>(E);

impl<E: Endpoint> Endpoint for BenchEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let start = Instant::now();
        let mut res = self.0.call(req).await?.into_response();
        res.headers_mut().insert(
            "x-request-id",
            "bench".parse().unwrap_or_else(|_| panic!("static header")),
        );
        res.headers_mut().insert(
            "x-response-time-ms",
            start
                .elapsed()
                .as_millis()
                .to_string()
                .parse()
                .unwrap_or_else(|_| panic!("static header")),
        );
        Ok(res)
    }
}

#[handler]
fn index() -> &'static str {
    "Hello, World!"
}

#[handler]
fn hello() -> &'static str {
    "Hello, World!"
}

#[handler]
fn json() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "message": "Hello, World!" }))
}

#[handler]
fn user(Path(id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": id }))
}

#[handler]
fn search(Query(params): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(&params).unwrap_or(serde_json::Value::Null))
}

#[handler]
fn echo(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(body)
}

#[handler]
fn payload(Path(size): Path<usize>) -> poem::Result<Json<serde_json::Value>> {
    if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
        return Err(poem::Error::from_status(poem::http::StatusCode::BAD_REQUEST));
    }
    Ok(Json(
        serde_json::json!({ "size": size, "data": "x".repeat(size) }),
    ))
}

#[handler]
fn cpu() -> Json<serde_json::Value> {
    let mut n: u64 = 0;
    for i in 0..200_000u64 {
        n = n.wrapping_add(i.wrapping_mul(i));
    }
    Json(serde_json::json!({ "result": n }))
}

#[handler]
fn sleepy() -> Json<serde_json::Value> {
    // Poem handlers are sync here; the sleep benchmark needs async.
    Json(serde_json::json!({ "slept_ms": 0 }))
}

#[handler]
async fn sleepy_async() -> Json<serde_json::Value> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Json(serde_json::json!({ "slept_ms": 1 }))
}

#[handler]
fn bad() -> poem::Result<Json<serde_json::Value>> {
    Err(poem::Error::from_status(poem::http::StatusCode::BAD_REQUEST))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/", get(index))
        .at("/json", get(json))
        .at("/hello", get(hello))
        .at("/users/:id", get(user))
        .at("/search", get(search))
        .at("/echo", post(echo))
        .at("/payload/:size", get(payload))
        .at("/cpu", get(cpu))
        .at("/sleep", get(sleepy_async))
        .at("/bad", get(bad));
    if std::env::var("MW").as_deref() == Ok("1") {
        println!("poem-server with middleware listening on 127.0.0.1:8080");
        Server::new(TcpListener::bind("127.0.0.1:8080"))
            .run(app.with(BenchMiddleware))
            .await
    } else {
        println!("poem-server listening on 127.0.0.1:8080");
        Server::new(TcpListener::bind("127.0.0.1:8080"))
            .run(app)
            .await
    }
}
