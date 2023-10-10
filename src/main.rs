use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

async fn main() {
    println!("main thread");
    http_server().await;
    println!("http_socket:");
}

#[actix_web::run]
async fn http_server() -> std::io::Result<()> {
    println!("Web server running at localhost:3000");

    // a thread pool is created here
    HttpServer::new(|| {
        println!("server threar");
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/hi", web::get().to(|| async { "Hello world!" }))
    })
    .bind(("127.0.0.1", 3000))?
    .workers(1)
    .run()
    .await
}
