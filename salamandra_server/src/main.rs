use std::{net::{TcpListener, TcpStream}, thread, io::{BufReader, BufRead, Write, Read}, fs::{self, File}};
use std::io::{ Result};
use std::str;

use encoding::{Encoding, EncoderTrap};
use encoding::all::ASCII;
use chrono::Utc;


fn main() {
    println!("Hello, world server!");
    start_server();


}

fn process_connection(mut stream: TcpStream){


    let mut buf_reader = BufReader::new(&stream);
    //thread::sleep(Duration::from_secs(10));
    let mut final_line = String::new();
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("index.html").unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    println!("termina cliente {:?}", stream)

}

fn encode_message_size(cmd: &str) -> Result<Vec<u8>> {
    let mut message_size = cmd.len();
    //println!("{:?}", cmd);
    message_size = message_size + 1;
    let message_size_str = message_size.to_string();
    let mut message_size_bytes = ASCII.encode(&message_size_str, EncoderTrap::Strict).map_err(|x| x.into_owned()).unwrap();
    message_size_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_size_bytes)
}

fn encode_message(cmd: &str) -> Result <Vec<u8>> {
    //println!("{:?}", cmd);
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII.encode(&message_str, EncoderTrap::Strict).map_err(|x| x.into_owned()).unwrap();
    message_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_bytes)
}

fn check_ack(mut ack_buf: &mut [u8]) -> String {

    let ack_slice: &str = str::from_utf8(&mut ack_buf).unwrap(); //string slice
    let mut ack_str = ack_slice.to_string(); //convert slice to string
    let index: usize = ack_str.rfind('\r').unwrap();
    //println!("{:?} server ACK:", ack_str.split_off(index));
    format!("{:?}", ack_str.split_off(index)); 
    if ack_str != "ACK"{
        //println!("received ACK from server");
        // end with error, maybe set a timeout
        return String::from("error")
    }
    String::from("ACK")
}

fn send_file_to_client(){
    println!("enviar binario a cliente");
    // conectarse al cliente
    // leer archivo
    // enviar archivo, escribir en el cliente

    
    let mut stream = TcpStream::connect("localhost:8081") // try!(TcpStream::connect(HOST));
    .expect("Couldn't connect to the server...");

    let full_path = String::from("./shared/principal.exe");


    let mut file = File::open(full_path).unwrap();
    let file_size = file.metadata().unwrap().len();
    println!("file_size {:?}", file_size);
    let mut ack_buf = [0u8; 8];


    //send file size
    let encoded_file_size = encode_message(&file_size.to_string()).unwrap();
    stream.write_all(&encoded_file_size).unwrap();

     //receive ack
     stream.read(&mut ack_buf).unwrap();
     if check_ack(&mut ack_buf) != "ACK" { println!("get_file ACK Failed"); }
     println!("[get_file]: received ACK from client [3]");

    let mut buf = [0u8; 8192];

    let mut remaining_data = file_size as i32;
    let ini = Utc::now();
    while remaining_data != 0 {

        if remaining_data >=8192 {

            let file_slab = file.read(&mut buf);
            match file_slab{
                Ok(n) => {
                    stream.write_all(&buf).unwrap();
                    remaining_data = remaining_data - n as i32;
                    println!("sent {} file bytes (big) | remaining: {}", n, remaining_data);

                }
                _ => {}
            }
        } else {
            let file_slab = file.read(&mut buf);
            match file_slab {
                //client must shrink this last buffer
                Ok(n) => {
                    stream.write_all(&buf).unwrap();
                    remaining_data = remaining_data - n as i32;
                    println!("sent {} file bytes (small) | remaining: {}", n, remaining_data);
                }
                _ => {}
            }
        }
        
    }
    let fin = Utc::now();
    println!("Tiempo transcurrido: {}", (fin.signed_duration_since(ini)));



}

fn start_server(){
    let listener = TcpListener::bind("127.0.0.1:8082").unwrap();
    println!("listening started, ready to accept");

    send_file_to_client();

    /*for stream in listener.incoming() {
        thread::spawn(|| {
            let mut stream = stream.unwrap();
            process_connection(stream);
        });
    }*/
}
