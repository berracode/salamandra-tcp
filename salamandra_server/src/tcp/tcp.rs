
#[allow(unused_imports)]

use std::{net::{TcpStream, Shutdown}, time::Duration, thread, io::{Write, Read}};
use std::{str, fs::{self, File}, error};
use encoding::{all::ASCII, EncoderTrap, Encoding};

use crate::{config::config::Config, file::file_manager};

const BASE_URL: &str = "http://192.168.1.161:8080/";

fn my_decode_message(buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message.chars().filter(|message_byte|{
        message_byte.is_ascii_graphic() == true 
    }).collect();

    clean_message
}

fn encode_message(cmd: &str) -> Result <Vec<u8>, Box<dyn error::Error + Send + Sync>> {
    //println!("{:?}", cmd);
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII.encode(&message_str, EncoderTrap::Strict).map_err(|x| x.into_owned())?;
    message_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_bytes)
}

/// Inicia difusión de la URL donde el cliente podrá descargar el archivo mediante HTTP
/// Valida que el archivo exista en ruta configurada en el server.toml
/// Una vez comprobemos que hay un archivo, leemos los clientes de file_data
/// enviar trama tcp con la url a cada cliente
/// recibir trama de confirmación cuando el cliente termine de descarga el archivo
/// responder a la CLI que ya el cliente x1 terminó
fn broadcast(mut stream: &TcpStream, config: Config){

    //validar que exista archivo en base_route
    let mut file_to_send = "";
    let mut fullpath = "".to_string();

    for entry in fs::read_dir(config.server.base_route.unwrap()).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            //clean path from file name:
            fullpath = String::from(entry.path().to_string_lossy());
            println!("fullpath {}",fullpath.clone() );
            let filename = String::from(str::replace(&fullpath.clone(), "./shared", ""));
            println!("file name {}", filename);
            let trimmed = &filename[1..];

            println!("trimed {}", trimmed);

            let mut file = File::open(&fullpath.clone()).unwrap();
            let file_size = file.metadata().unwrap().len();

            //println!("{}  [{:?} bytes]", style(trimmed).green(), style(file_size).cyan());
            //format data:
            let partial = format!("{}  [{:?} bytes]", trimmed, file_size);
            println!("{:?}", partial);


            /*for c in partial.chars()
            {
                //load the buffer
                ls_bytes.push(c as u8);
            }
            ls_bytes.push('\n' as u8);*/
        }
    }
    file_to_send = &fullpath[2..];

    println!("file_to_send {}", file_to_send);

    let mut url = format!("{BASE_URL}{file_to_send}");
    println!("url {url}");

    //leer clientes del file_data
    let mut clients = file_manager::get_all_clients().unwrap();
    let mut buf = [0 as u8; 20];

    for client in clients {
        println!("Hay que hacer la conexión tcp en hilos, hacia los clientes tcp que se registraron en el server
        pero con fines de testing se lo enviare al mismo que me envia");


        stream.write_all(url.as_bytes()).unwrap();
        println!("[SERVER] URL base enviada a cliente {client}");

        loop{
            stream.read(&mut buf).unwrap();
            if buf.len() > 0 {
                let msg = my_decode_message(&mut buf);
                println!("msg in loop fake {}", msg );

                break;
            }
        }

        //respuesta CLI
        stream.write_all(b"CLI el cliente descargo el archivo").unwrap();  
        

    }




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

    } else if msg.as_bytes() == "broadcast".as_bytes() {
        broadcast(&stream, config);
        response = "OK";
        
    }

   
    stream.write_all(response.as_bytes()).unwrap();
    println!("A client has been finished {:?}", stream)
   

}



#[cfg(test)]
mod tests {
    use std::{io::Read, str::from_utf8};

    use super::*;

    #[test]
    fn connection_broadcats() {
        let mut stream = TcpStream::connect("192.168.1.161:9123").unwrap();
        let msg = b"broadcast";

        stream.write(msg).unwrap();

        let mut response_data = [0 as u8; 47];
        let mut response = 0;

        match stream.read_exact(&mut response_data) {
            Ok(_) => {
                if response_data.starts_with(b"http") {
                    let text = from_utf8(&response_data).unwrap();
                    println!("text {text}");
                    println!("INiciando descarga...");
                    thread::sleep(Duration::from_secs(10));
                    stream.write(b"ok").unwrap();
                    response_data.fill_with(Default::default);
                    println!("response_data {:?}", response_data);

                    stream.read(&mut response_data).unwrap();
                    let text = from_utf8(&response_data).unwrap();
                    println!("text 2 {text}");
                    
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
