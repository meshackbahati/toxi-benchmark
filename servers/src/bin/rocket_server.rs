//! Rocket target: full benchmark matrix (A–G, K–N).

#[macro_use]
extern crate rocket;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Data, Request, Response};
use std::time::Instant;

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[get("/json")]
fn json() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "message": "Hello, World!" }))
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, World!"
}

#[derive(rocket::serde::Serialize)]
struct User {
    id: String,
}

#[get("/users/<id>")]
fn user(id: &str) -> Json<User> {
    Json(User { id: id.to_string() })
}

#[get("/search?<q>&<page>&<limit>")]
fn search(q: &str, page: &str, limit: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "q": q, "page": page, "limit": limit }))
}

#[post("/echo", data = "<body>")]
fn echo(body: Json<serde_json::Value>) -> Json<serde_json::Value> {
    body
}

#[get("/payload/<size>")]
fn payload(size: usize) -> Result<Json<serde_json::Value>, rocket::http::Status> {
    if !matches!(size, 1024 | 10240 | 102400 | 1048576) {
        return Err(rocket::http::Status::BadRequest);
    }
    Ok(Json(
        serde_json::json!({ "size": size, "data": "x".repeat(size) }),
    ))
}

#[get("/cpu")]
fn cpu() -> Json<serde_json::Value> {
    let mut n: u64 = 0;
    for i in 0..200_000u64 {
        n = n.wrapping_add(i.wrapping_mul(i));
    }
    Json(serde_json::json!({ "result": n }))
}

#[get("/sleep")]
async fn sleepy() -> Json<serde_json::Value> {
    rocket::tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Json(serde_json::json!({ "slept_ms": 1 }))
}

#[get("/bad")]
fn bad() -> Status {
    Status::BadRequest
}

use rocket::http::Status;
use rocket::serde::json::Json;

/// Timing plus request-id fairing, enabled with MW=1.
struct BenchFairing;

#[rocket::async_trait]
impl Fairing for BenchFairing {
    fn info(&self) -> Info {
        Info {
            name: "bench timing and request id",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        req.local_cache(|| Instant::now());
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let ms = req.local_cache(Instant::now).elapsed().as_millis();
        res.set_header(Header::new("x-request-id", "bench"));
        res.set_header(Header::new("x-response-time-ms", ms.to_string()));
    }
}

#[launch]
fn rocket() -> _ {
    println!("rocket-server listening on 127.0.0.1:8080");
    let r = rocket::build()
        .configure(rocket::Config {
            port: 8080,
            address: "127.0.0.1".parse().unwrap(),
            ..rocket::Config::default()
        })
        .mount(
            "/",
            routes![index, json, hello, user, search, echo, payload, cpu, sleepy, bad],
        );
    if std::env::var("MW").as_deref() == Ok("1") {
        r.attach(BenchFairing)
    } else {
        r
    }
}
