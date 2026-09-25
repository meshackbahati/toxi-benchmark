//! Salvo target: full benchmark matrix (A–G, K–N).

use salvo::prelude::*;
use std::time::Instant;

#[handler]
async fn index(res: &mut Response) {
    res.render(Text::Plain("Hello, World!"));
}

#[handler]
async fn hello(res: &mut Response) {
    res.render(Text::Plain("Hello, World!"));
}

#[handler]
async fn json(res: &mut Response) {
    res.render(Json(serde_json::json!({ "message": "Hello, World!" })));
}

#[handler]
async fn user(req: &mut Request, res: &mut Response) {
    let id: String = req.param("id").unwrap_or_default();
    res.render(Json(serde_json::json!({ "id": id })));
}

#[handler]
async fn search(req: &mut Request, res: &mut Response) {
    let q: String = req.query("q").unwrap_or_default();
    let page: String = req.query("page").unwrap_or_default();
    let limit: String = req.query("limit").unwrap_or_default();
    res.render(Json(
        serde_json::json!({ "q": q, "page": page, "limit": limit }),
    ));
}

#[handler]
async fn echo(req: &mut Request, res: &mut Response) {
    let body: serde_json::Value = req.parse_json().await.unwrap_or(serde_json::Value::Null);
    res.render(Json(body));
}

#[handler]
async fn payload(req: &mut Request, res: &mut Response) {
    let size: usize = req.param("size").unwrap_or(0);
    if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }
    res.render(Json(
        serde_json::json!({ "size": size, "data": "x".repeat(size) }),
    ));
}

#[handler]
async fn cpu(res: &mut Response) {
    let mut n: u64 = 0;
    for i in 0..200_000u64 {
        n = n.wrapping_add(i.wrapping_mul(i));
    }
    res.render(Json(serde_json::json!({ "result": n })));
}

#[handler]
async fn sleepy(res: &mut Response) {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    res.render(Json(serde_json::json!({ "slept_ms": 1 })));
}

#[handler]
async fn bad(res: &mut Response) {
    res.status_code(StatusCode::BAD_REQUEST);
}

/// Timing plus request-id hoop, enabled with MW=1.
#[handler]
async fn bench_mw(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let start = Instant::now();
    ctrl.call_next(req, depot, res).await;
    res.headers_mut()
        .insert("x-request-id", "bench".parse().unwrap());
    res.headers_mut().insert(
        "x-response-time-ms",
        start
            .elapsed()
            .as_millis()
            .to_string()
            .parse()
            .unwrap_or_else(|_| "0".parse().unwrap()),
    );
}

fn router() -> Router {
    Router::new()
        .push(Router::with_path("/").get(index))
        .push(Router::with_path("json").get(json))
        .push(Router::with_path("hello").get(hello))
        .push(Router::with_path("users/{id}").get(user))
        .push(Router::with_path("search").get(search))
        .push(Router::with_path("echo").post(echo))
        .push(Router::with_path("payload/{size}").get(payload))
        .push(Router::with_path("cpu").get(cpu))
        .push(Router::with_path("sleep").get(sleepy))
        .push(Router::with_path("bad").get(bad))
}

#[tokio::main]
async fn main() {
    let acceptor = TcpListener::new("127.0.0.1:8080").bind().await;
    if std::env::var("MW").as_deref() == Ok("1") {
        println!("salvo-server with middleware listening on 127.0.0.1:8080");
        Server::new(acceptor)
            .serve(Router::new().hoop(bench_mw).push(router()))
            .await;
    } else {
        println!("salvo-server listening on 127.0.0.1:8080");
        Server::new(acceptor).serve(router()).await;
    }
}
