use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

fn query_block_by_hash(_path: web::Path<(u32, String)>) -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

fn query_block_by_number(_path: web::Path<u32>) -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

fn query_tx_by_id(_path: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

fn query(_: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

fn invoke(_: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

fn pong(_: HttpRequest) -> impl Responder {
    "pong"
}

pub fn main() {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/block")
                    .route("/hash/{hash}", web::get().to(query_block_by_hash))
                    .route("/number/{number}", web::get().to(query_block_by_number)),
            )
            .service(web::scope("/transaction").route("/id/{id}", web::get().to(query_tx_by_id)))
            .service(
                web::scope("/contract")
                    .route("/query", web::post().to(query))
                    .route("/invoke", web::post().to(invoke)),
            )
            .route("/ping", web::get().to(pong))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
