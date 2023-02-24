
#[allow(unused_imports)]

use std::{net::{TcpStream, Shutdown}, time::Duration, thread, io::{Write, Read}};
use std::str;
use crate::{config::config::Config, file::file_manager};

fn my_decode_message(mut buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message.chars().filter(|message_byte|{
        message_byte.is_ascii_graphic() == true 
    }).collect();

    clean_message
}

/// Inicia difusión de la URL donde el cliente podrá descargar el archivo mediante HTTP
/// Valida que el archivo exista en ruta configurada en el server.toml
/// Una vez comprobemos que hay un archivo, leemos los clientes de file_data
/// enviar trama tcp con la url a cada cliente
/// recibir trama de confirmación cuando el cliente termine de descarga el archivo
/// responder a la CLI que ya el cliente x1 terminó
fn broadcast(){

}


pub fn process_connection(mut stream: TcpStream, config: Config){
    println!("New client connected from {:?}", stream);

    let new_client = stream.peer_addr().unwrap();
    let mut buf = [0 as u8; 20];


    stream.read(&mut buf).unwrap();
    let msg = my_decode_message(&mut buf);
    let mut response = "NO";

    if msg.as_bytes() == "client-record".as_bytes() {
        // debemos escribir en archivo de texto al cliente registrado. y responder registro exitoso.
        println!("Saving client in file...");
        let is_saved =  file_manager::save_client(new_client);

        if is_saved == 1 {
            response = "OK";
        }

    } else if msg.as_bytes() == "broadcast" {


        
    }

   
    stream.write_all(response.as_bytes()).unwrap();
    println!("A client has been finished {:?}", stream)
   

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
