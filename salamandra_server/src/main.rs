
use actix_files::NamedFile;
use salamandra_server::config::config::Config;
use salamandra_server::tcp::tcp;
#[allow(unused_imports)]

use std::{net::{TcpListener, TcpStream}, thread, io::{ Write}};
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config= Config::new();
    let ip = config.get_ip();
    let _port = config.get_port();

    thread::spawn(move|| {
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

    HttpServer::new(|| {
        App::new()
            .route("/shared/{filename:.*}", web::get().to(index))
    })
    .bind((ip, 8080))?
    .run()
    .await
}


#[cfg(test)]
mod tests {
    use std::{io::Read, str::from_utf8};

    use super::*;

    #[test]
    fn connection_client_record() {
        let mut stream = TcpStream::connect("192.168.1.161:9123").unwrap();
        let msg = b"client-record";

        stream.write(msg).unwrap();

        let mut response_data = [0 as u8; 2];
        let mut response = 0;

        match stream.read_exact(&mut response_data) {
            Ok(_) => {
                if &response_data == "OK".as_bytes() {
                    println!("Reply is ok!");
                    response = 1;
                } else {
                    let text = from_utf8(&response_data).unwrap();
                    println!("Unexpected reply: {}", text);
                }
            },
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }


        assert_eq!(1, response);
    }

}

