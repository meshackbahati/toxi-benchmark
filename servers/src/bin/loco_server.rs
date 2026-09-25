//! Loco target: full benchmark matrix (A–G, K–N).
//!
//! Minimal application: no database, no workers. Configuration comes from
//! `config/development.yaml` relative to the working directory; invoke
//! with the `servers/` directory as cwd. MW=1 layers axum timing plus
//! request-id middleware over the JSON route for the overhead benchmark.

use async_trait::async_trait;
use axum::extract::State;
use axum::response::Response;
use loco_rs::{
    app::{AppContext, Hooks},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    controller::{format, AppRoutes, Routes},
    environment::Environment,
    task::Tasks,
    Result,
};
use std::collections::HashMap;
use std::time::Instant;

async fn json_handler(State(_ctx): State<AppContext>) -> Result<Response> {
    Ok(format::json(
        serde_json::json!({ "message": "Hello, World!" }),
    )?)
}

async fn hello_handler(State(_ctx): State<AppContext>) -> Result<Response> {
    use axum::response::IntoResponse;
    Ok(axum::response::Html("Hello, World!").into_response())
}

async fn user_handler(
    State(_ctx): State<AppContext>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Response> {
    Ok(format::json(serde_json::json!({ "id": id }))?)
}

async fn search_handler(
    State(_ctx): State<AppContext>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Response> {
    Ok(format::json(
        serde_json::to_value(&params).unwrap_or(serde_json::Value::Null),
    )?)
}

async fn echo_handler(
    State(_ctx): State<AppContext>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> Result<Response> {
    Ok(format::json(body)?)
}

async fn payload_handler(
    State(_ctx): State<AppContext>,
    axum::extract::Path(size): axum::extract::Path<usize>,
) -> Result<Response> {
    if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
        return Err(loco_rs::errors::Error::BadRequest(
            "size must be 1024, 10240, 102400, or 1048576".to_string(),
        ));
    }
    Ok(format::json(
        serde_json::json!({ "size": size, "data": "x".repeat(size) }),
    )?)
}

async fn cpu_handler(State(_ctx): State<AppContext>) -> Result<Response> {
    let mut n: u64 = 0;
    for i in 0..200_000u64 {
        n = n.wrapping_add(i.wrapping_mul(i));
    }
    Ok(format::json(serde_json::json!({ "result": n }))?)
}

async fn sleep_handler(State(_ctx): State<AppContext>) -> Result<Response> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Ok(format::json(serde_json::json!({ "slept_ms": 1 }))?)
}

async fn bad_handler(State(_ctx): State<AppContext>) -> Result<Response> {
    Err(loco_rs::errors::Error::BadRequest(
        "bad request".to_string(),
    ))
}

async fn bench_mw(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    use axum::response::IntoResponse;
    let start = Instant::now();
    let mut res = next.run(req).await.into_response();
    res.headers_mut().insert(
        "x-request-id",
        "bench".parse().unwrap_or(axum::http::HeaderValue::from_static("0")),
    );
    res.headers_mut().insert(
        "x-response-time-ms",
        start
            .elapsed()
            .as_millis()
            .to_string()
            .parse()
            .unwrap_or(axum::http::HeaderValue::from_static("0")),
    );
    res
}

struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        "bench"
    }

    fn app_version() -> String {
        "0.1.0".to_string()
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: loco_rs::config::Config,
    ) -> Result<BootResult> {
        create_app::<Self>(mode, environment, config).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        use axum::routing::{get, post};
        let json_route = get(json_handler);
        let json_route = if std::env::var("MW").as_deref() == Ok("1") {
            json_route.layer(axum::middleware::from_fn(bench_mw))
        } else {
            json_route
        };
        AppRoutes::empty().add_routes(vec![
            Routes::new().add("/", get(hello_handler)),
            Routes::new().add("/json", json_route),
            Routes::new().add("/hello", get(hello_handler)),
            Routes::new().add("/users/{id}", get(user_handler)),
            Routes::new().add("/search", get(search_handler)),
            Routes::new().add("/echo", post(echo_handler)),
            Routes::new().add("/payload/{size}", get(payload_handler)),
            Routes::new().add("/cpu", get(cpu_handler)),
            Routes::new().add("/sleep", get(sleep_handler)),
            Routes::new().add("/bad", get(bad_handler)),
        ])
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    loco_rs::cli::main::<App>().await
}
