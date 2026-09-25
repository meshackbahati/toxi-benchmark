//! Warp target: full benchmark matrix (A–G, K–N).
//!
//! The middleware run layers request timing plus a static request id over
//! the same routes (documented difference: the id is static because warp
//! response headers take fixed values at filter construction).

use std::time::Instant;
use warp::{Filter, Reply};

fn reply_json(
    value: serde_json::Value,
) -> impl warp::Reply {
    warp::reply::json(&value)
}

#[tokio::main]
async fn main() {
    let index = warp::path::end()
        .and(warp::get())
        .map(|| "Hello, World!");
    let hello = warp::path("hello")
        .and(warp::get())
        .map(|| "Hello, World!");
    let json = warp::path("json").and(warp::get()).map(|| {
        reply_json(serde_json::json!({ "message": "Hello, World!" }))
    });
    let user = warp::path!("users" / String).and(warp::get()).map(|id: String| {
        reply_json(serde_json::json!({ "id": id }))
    });
    let search = warp::path("search")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(|q: std::collections::HashMap<String, String>| {
            reply_json(serde_json::to_value(&q).unwrap_or(serde_json::Value::Null))
        });
    let echo = warp::path("echo")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: serde_json::Value| reply_json(body));
    let payload = warp::path!("payload" / usize).and(warp::get()).map(|size: usize| {
        if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
            return warp::reply::with_status(
                "bad request",
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
        warp::reply::json(&serde_json::json!({ "size": size, "data": "x".repeat(size) }))
            .into_response()
    });
    let cpu = warp::path("cpu").and(warp::get()).map(|| {
        let mut n: u64 = 0;
        for i in 0..200_000u64 {
            n = n.wrapping_add(i.wrapping_mul(i));
        }
        reply_json(serde_json::json!({ "result": n }))
    });
    let sleepy = warp::path("sleep").and(warp::get()).and_then(|| async {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        Ok::<_, std::convert::Infallible>(reply_json(
            serde_json::json!({ "slept_ms": 1 }),
        ))
    });
    let bad = warp::path("bad").and(warp::get()).map(|| {
        warp::reply::with_status("bad request", warp::http::StatusCode::BAD_REQUEST)
    });
    let routes = index
        .or(hello)
        .or(json)
        .or(user)
        .or(search)
        .or(echo)
        .or(payload)
        .or(cpu)
        .or(sleepy)
        .or(bad);

    if std::env::var("MW").as_deref() == Ok("1") {
        println!("warp-server with middleware listening on 127.0.0.1:8080");
        let timed = warp::any()
            .map(|| Instant::now())
            .and(routes)
            .map(|start: Instant, reply: _| {
                warp::reply::with_header(
                    warp::reply::with_header(
                        reply,
                        "x-request-id",
                        "bench",
                    ),
                    "x-response-time-ms",
                    start.elapsed().as_millis().to_string(),
                )
            });
        // Logging is attached for timing parity with the other stacks.
        let logged = timed.with(warp::log::custom(|_| {}));
        warp::serve(logged).run(([127, 0, 0, 1], 8080)).await;
    } else {
        println!("warp-server listening on 127.0.0.1:8080");
        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    }
}
