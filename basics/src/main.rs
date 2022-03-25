#[macro_use]
extern crate actix_web;
use std::{env, io};

use actix_files as fs;
use actix_session::{CookieSession, Session};
use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Result
};
use json::JsonValue;
use lazy_static::lazy_static;

mod router;
mod model;
mod common;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    // let conn = model::connect_mysql();
    // lazy_static! {
    //     static ref POOL: Pool = {
    //         let db_url = env::var("DATABASE_URL").expect("Database url not set");
    //         let manager = ConnectionManager::<PgConnection>::new(db_url);
    //         Pool::new(manager).expect("Failed to create db pool")
    //     };
    // }


    HttpServer::new(|| {
        App::new()
            // cookie session middleware
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register favicon
            .service(router::favicon)
            // register simple route, handle all methods
            .service(router::welcome)
            // with path parameters
            .service(web::resource("/user/{name}").route(web::get().to(router::with_param)))
            .service(web::resource("/user_all/{name}").route(web::get().to(router::with_param)))
            .service(web::resource("/sql").route(web::get().to(|| async {
                // const res = model::query_map(conn);
                return HttpResponse::Ok().json("")
            })))
            .service(web::resource("/json").route(web::post().to(router::index_mjsonrust)))
            .service(web::resource("/json-self").route(web::get().to(router::index_mjsonrust_self)))
            // async response body
            .service(
                web::resource("/async-body/{name}").route(web::get().to(router::response_body)),
            )
            .service(
                web::resource("/test").to(|req: HttpRequest| match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    Method::POST => HttpResponse::MethodNotAllowed(),
                    _ => HttpResponse::NotFound(),
                }),
            )
            .service(web::resource("/error").to(|| async {
                error::InternalError::new(
                    io::Error::new(io::ErrorKind::Other, "test"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }))
            // static files
            .service(fs::Files::new("/static", "static").show_files_listing())
            // redirect
            .service(web::resource("/").route(web::get().to(|req: HttpRequest| async move {
                println!("{:?}", req);
                HttpResponse::Found()
                    .insert_header((header::LOCATION, "static/welcome.html"))
                    .finish()
            })))
            // default
            .default_service(web::to(router::default_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
