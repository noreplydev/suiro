use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Web server running at localhost:8080");

    // a thread pool is created here
    HttpServer::new(|| {
        println!("server threar");
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/hi", web::get().to(|| async { "Hello world!" }))
    })
    .bind(("127.0.0.1", 8080))?
    .workers(1)
    .run()
    .await
}
