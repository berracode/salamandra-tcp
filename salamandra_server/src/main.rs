use actix_files::NamedFile;
use salamandra_server::config::config::Config;
use std::{net::{TcpListener, TcpStream}, thread, io::{BufReader, Write},  time::Duration};
use std::io::{ Result};
use actix_web::{HttpServer, App, Responder, HttpResponse, get, web, HttpRequest};




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


fn process_connection(mut stream: TcpStream, config: Config){
    println!("un nuevo cliente conectado desde {:?}", stream);
    thread::sleep(Duration::from_secs(10));
    let response = "HOLA CLIENTE";
    stream.write_all(response.as_bytes()).unwrap();
    println!("chao {:?}", stream)
   

}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config= Config::new();
    let ip = config.get_ip();
    let port = config.get_port();

    thread::spawn(move|| {
        let listener = TcpListener::bind(config.get_listener()).unwrap();
    
        println!("listening started, ready to accept");
        for stream in listener.incoming() {
            let config = config.clone();

            thread::spawn(|| {
                let stream = stream.unwrap();
                process_connection(stream, config);
            });
        }
    
    });

    HttpServer::new(|| {
        App::new()
            .route("/shared/{filename:.*}", web::get().to(index))
    })
    .bind((ip, port))?
    .run()
    .await
}



