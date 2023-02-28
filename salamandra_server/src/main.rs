use actix_files::NamedFile;
use salamandra_server::config::config::Config;
use salamandra_server::tcp::tcp;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::io::Result;
use std::{
    net::{TcpListener},
    thread,
};


async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path = ".".to_string();
    path.push_str(req.path());

    let file = NamedFile::open(path).unwrap();

    Ok(file)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new();
    let ip = config.get_ip();
    let _port = config.get_port();

    start_tcp_server(config);

    HttpServer::new(|| App::new().route("/shared/{filename:.*}", web::get().to(index)))
        .bind((ip, 8080))?
        .run()
        .await
}

fn start_tcp_server(config: Config) {
    thread::spawn(move || {
        let listener = TcpListener::bind(config.get_listener()).unwrap();

        println!("listening started, ready to accept");
        for stream in listener.incoming() {
            let config = config.clone();

            thread::spawn(|| {
                let stream = stream.unwrap();
                tcp::process_connection(stream, config);
            });
        }
    });
}
