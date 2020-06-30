mod endpoints;

use actix_web::{HttpServer, App, web, HttpResponse, guard, middleware};
use actix_web::guard::Guard;
use actix_web::dev::RequestHead;
use crate::endpoints::{multipart_handler, json_handler};

async fn index() -> HttpResponse {
    //TODO: static html
    HttpResponse::Ok().body("index")
}

struct MultipartHeaderGuard {}

impl Guard for MultipartHeaderGuard {
    fn check(&self, request: &RequestHead) -> bool {
        if let Some(content_type) = request.headers.get("content-type") {
            return content_type.to_str().map_or(false, |h| h.starts_with("multipart/form-data;"));
        }
        false
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    std::fs::create_dir_all("images").unwrap();

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default())
            .service(
                web::resource("/")
                    .route(
                        web::get().to(index)
                    )
                    .route(
                        web::post()
                            .guard(guard::Header("content-type", "application/json"))
                            .to(json_handler)
                    )
                    .route(
                        web::post()
                            .guard(MultipartHeaderGuard {})
                            .to(multipart_handler)
                    )
            )
    })
        .bind("0.0.0.0:8088")?
        .run()
        .await
}
