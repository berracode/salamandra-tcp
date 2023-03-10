use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufRead,
    net::SocketAddr,
    str::{self, FromStr},
    sync::mpsc,
};
#[allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    thread,
    time::Duration,
};

use crate::{
    config::config::Config,
    file::file_manager,
    tcp::connection::{self, Connection},
};

const BASE_URL: &str = "http://192.168.1.161:8080/";

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    operation: String,
    data: Option<String>,
}

enum Operation {
    ClientRecord,
    Broadcast,
}

/// Inicia difusión de la URL donde el cliente podrá descargar el archivo mediante HTTP
/// Valida que el archivo exista en ruta configurada en el server.toml
/// Una vez comprobemos que hay un archivo, leemos los clientes de file_data
/// enviar trama tcp con la url a cada cliente
/// recibir trama de confirmación cuando el cliente termine de descarga el archivo
/// responder a la CLI que ya el cliente x1 terminó
fn broadcast(connection: &Connection) {
    let base_route = connection.base_route().unwrap();

    //validar que exista archivo en base_route
    let mut file_to_send = String::new();
    let mut fullpath = String::new();

    for entry in fs::read_dir(base_route).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            //clean path from file name:
            fullpath = String::from(entry.path().to_string_lossy());
            println!("fullpath {}", fullpath.clone());
            let filename = String::from(str::replace(&fullpath.clone(), "./shared", ""));
            println!("file name {}", filename);
            let trimmed = &filename[1..];

            println!("trimed {}", trimmed);

            let file = File::open(&fullpath.clone()).unwrap();
            let file_size = file.metadata().unwrap().len();

            //println!("{}  [{:?} bytes]", style(trimmed).green(), style(file_size).cyan());
            //format data:
            let partial = format!("{}  [{:?} bytes]", trimmed, file_size);
            println!("{:?}", partial);
        }
    }
    file_to_send = (&fullpath[2..]).to_string();

    println!("file_to_send {}", file_to_send);

    let url = format!("{BASE_URL}{file_to_send}");
    println!("url {url}");

    //leer clientes del file_data
    let clients = file_manager::get_all_clients().unwrap();
    let config = connection.config.clone();

    let mut threads = vec![];
    let (tx, rx) = mpsc::channel();

    for client in clients.clone() {
        let url_clone = url.clone();
        let config = config.clone();
        let tx_n = tx.clone();

        let thread = thread::spawn(move || {
            let stream = TcpStream::connect(client).unwrap(); //connection to the client
            let mut connection = Connection::new(stream, config);
            connection.send_message(&url_clone); //send url to client
            let mut response_byte = connection.read_message().unwrap().to_vec();
            connection.reader.consume(response_byte.len());
            let response_from_client = connection::decode_message(&mut response_byte);
            println!("response_from_client: {}", response_from_client);

            if response_from_client.eq_ignore_ascii_case("downloaded") {
                println!("Termina {:?}", connection.stream);
                tx_n.send(connection.stream).unwrap();
                drop(tx_n); 
            }

            println!("termina .. {}", client);
        });

        threads.push(thread);
    }

    /*while threads.len() > 0 {
        let cur_thread = threads.remove(0); // moves it into cur_thread

        while let Ok(received) = rx.recv() {
            println!("termina: {:?}", received);
        }
        cur_thread.join().unwrap();
    }*/
    for i in clients {
        println!("termina: {:?}, {}", rx.recv(), i);
    }
   

    println!("fin BROACASTING...")
}

pub fn process_connection(stream: TcpStream, config: Config) {
    println!("New client connected from {:?}", stream);

    let mut connection = Connection::new(stream, config);

    let mut buf_vec: Vec<u8> = connection.read_message().unwrap().to_vec();
    connection.reader.consume(buf_vec.len());
    let request = connection::decode_message(&mut buf_vec);

    let req: Request = serde_json::from_str(&request).unwrap();

    println!("req: {:?}", req);
    let mut response = "NO";

    match parse_operation(&req.operation) {
        Some(Operation::ClientRecord) => {
            println!("Saving client in file...");
            let new_client = SocketAddr::from_str(req.data.unwrap().as_str()).unwrap();
            let is_saved = file_manager::save_client(new_client);

            if is_saved == 1 {
                response = "OK";
            }
        }
        Some(Operation::Broadcast) => {
            broadcast(&connection);
            response = "OK";
        }
        None => {
            println!("Invalid operation: {}", req.operation);
            response = "ERROR";
        }
    }

    connection.send_message(response);
    println!("A client has been finished {:?}", connection.stream)
}

fn parse_operation(operation_str: &str) -> Option<Operation> {
    match operation_str.to_lowercase().as_str() {
        "client-record" => Some(Operation::ClientRecord),
        "broadcast" => Some(Operation::Broadcast),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, str::from_utf8};

    use serde_json::json;

    use super::*;

    #[test]
    fn connection_broadcats_from_cli() {
        let stream = TcpStream::connect("192.168.1.161:9123").unwrap();
        let config = Config::new();

        let mut connection = Connection::new(stream, config);

        let request = json!({
            "operation": "broadcast",
            "data":"null"
        });
        connection.send_message(request.to_string().as_str()); //cli manda el mensaje broacdast al server

        let mut response_byte = connection.read_message().unwrap().to_vec();
        connection.reader.consume(response_byte.len());
        let response_from_server = connection::decode_message(&mut response_byte);
        println!("response_from_server: {}", response_from_server);

        assert_eq!(String::from("OK"), response_from_server);
    }

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
            }
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }

        assert_eq!(1, response);
    }
}
