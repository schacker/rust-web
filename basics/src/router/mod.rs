use std::convert::Infallible;

use actix_files as fs;
use actix_session::{CookieSession, Session};
use actix_utils::mpsc;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Result, Responder, Either
};
use bytes::Bytes;
use async_stream::*;
use fs::NamedFile;
use futures::StreamExt;
use json::JsonValue;
use serde::{Serialize, Deserialize};
use serde_json::json;

/// favicon handler
#[get("/favicon")]
pub async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("../static/favicon.ico")?)
}

/// simple index handler9
#[get("/welcome")]
pub async fn welcome(session: Session, req: HttpRequest) -> HttpResponse {
    println!("{:?}", req);

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter").unwrap() {
        println!("SESSION value: {}", count);
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter).unwrap();

    // response
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/welcome.html"))
}

/// 404 handler
pub async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("../static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

/// response body
pub async fn response_body(path: web::Path<String>) -> HttpResponse {
    // let text = format!("Hello {}!", *path);

    // let (tx, rx_body) = mpsc::channel();
    // let _ = tx.send(Ok::<_, Error>(Bytes::from(text)));

    // HttpResponse::Ok().streaming(rx_body)
    let name = path.into_inner();

    HttpResponse::Ok().streaming(stream! {
        yield Ok::<_, Infallible>(web::Bytes::from("Hello "));
        yield Ok::<_, Infallible>(web::Bytes::from(name));
        yield Ok::<_, Infallible>(web::Bytes::from("!"));
    })
}

/// handler with path parameters like `/user/{name}/`
pub async fn with_param(req: HttpRequest, path: web::Path<(String,)>) -> HttpResponse {
  println!("{:?}", req);

  HttpResponse::Ok()
      .content_type("text/plain")
      .body(format!("Hello {:?}!", path.0))
}
pub async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

/// This handler uses json extractor
async fn index(item: web::Json<MyObj>) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

/// This handler uses json extractor with limit
async fn extract_item(item: web::Json<MyObj>, req: HttpRequest) -> HttpResponse {
    println!("request: {:?}", req);
    println!("model: {:?}", item);

    HttpResponse::Ok().json(item.0) // <- send json response
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// This handler manually load request payload and parse json object
async fn index_manual(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<MyObj>(&body)?;
    Ok(HttpResponse::Ok().json(obj)) // <- send response
}

/// This handler manually load request payload and parse json-rust
pub async fn index_mjsonrust(body: web::Bytes) -> Result<HttpResponse, Error> {
    // body is loaded, now we can deserialize json-rust
    let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    let self_json = json!({
        "code": 10000,
        "data": {
            "somekey": "i am key",
            "theme": [{
                "list": [1,2,3]
            }]
        },
        "message": ""
    });
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(injson.dump()))
}

pub async fn index_mjsonrust_self(body: web::Bytes) -> Result<HttpResponse, Error> {
    // body is loaded, now we can deserialize json-rust
    let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    let self_json = json!({
        "code": 10000,
        "data": {
            "somekey": "i am key",
            "theme": [{
                "list": [1,2,3]
            }]
        },
        "message": ""
    });
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(self_json.to_string()))
}