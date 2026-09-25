//! Toxi target: full benchmark matrix (A–G, K–N).
//!
//! MW=1 serves the same routes behind request-id plus metrics middleware
//! for the middleware-overhead benchmark.

use toxi::middleware::{Metrics, RequestIdMiddleware};
use toxi::prelude::*;
use toxi::json_response;

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, World!"))
}

async fn json(_req: Request) -> Result<Response> {
    Ok(json_response!({ "message": "Hello, World!" }))
}

async fn user(Path(params): Path<serde_json::Value>) -> Result<Response> {
    let id = params
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    Ok(Response::json(serde_json::json!({ "id": id })))
}

async fn search(Query(params): Query<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(params))
}

async fn echo(Json(body): Json<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(body))
}

async fn payload(Path(params): Path<serde_json::Value>) -> Result<Response> {
    let size = params
        .get("size")
        .and_then(|v| v.as_str().unwrap_or("").parse::<usize>().ok())
        .or_else(|| {
            params
                .get("size")
                .and_then(|v| v.as_u64().map(|n| n as usize))
        })
        .unwrap_or(0);
    if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
        return Err(Error::BadRequest(
            "size must be 1024, 10240, 102400, or 1048576".to_string(),
        ));
    }
    Ok(Response::json(
        serde_json::json!({ "size": size, "data": "x".repeat(size) }),
    ))
}

async fn cpu(_req: Request) -> Result<Response> {
    let mut n: u64 = 0;
    for i in 0..200_000u64 {
        n = n.wrapping_add(i.wrapping_mul(i));
    }
    Ok(Response::json(serde_json::json!({ "result": n })))
}

async fn sleepy(_req: Request) -> Result<Response> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Ok(Response::json(serde_json::json!({ "slept_ms": 1 })))
}

async fn bad(_req: Request) -> Result<Response> {
    Err(Error::BadRequest("bad request".to_string()))
}

fn build_router() -> Router {
    let mut router = Router::new();
    router.get("/", hello);
    router.get("/json", json);
    router.get("/hello", hello);
    router.get("/users/:id", user);
    router.get("/search", search);
    router.post("/echo", echo);
    router.get("/payload/:size", payload);
    router.get("/cpu", cpu);
    router.get("/sleep", sleepy);
    router.get("/bad", bad);
    router
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
    if std::env::var("MW").as_deref() == Ok("1") {
        println!("toxi-server with middleware listening on {addr}");
        let svc = RequestIdMiddleware::new(Metrics::new(build_router()));
        Server::new(svc).listen(addr).await
    } else {
        println!("toxi-server listening on {addr}");
        Server::new(build_router()).listen(addr).await
    }
}
