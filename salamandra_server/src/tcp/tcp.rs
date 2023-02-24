
#[allow(unused_imports)]

use std::{net::{TcpStream, Shutdown}, time::Duration, thread, io::{Write, Read}};
use std::str;
use crate::config::config::Config;

#[allow(unused_mut)]
fn my_decode_message(mut buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message.chars().filter(|message_byte|{
        message_byte.is_ascii_graphic() == true 
    }).collect();

    clean_message
}

#[allow(unused_variables)]
pub fn process_connection(mut stream: TcpStream, config: Config){
    println!("New client connected from {:?}", stream);

    let mut buf = [0 as u8; 20];


    stream.read(&mut buf).unwrap();
    let msg = my_decode_message(&mut buf);
    let mut response = "12";

    if msg.as_bytes() == "client-record".as_bytes() {
        // debemos escribir en archivo de texto al cliente registrado. y responder registro exitoso.
        println!("Saving client in file...");
        thread::sleep(Duration::from_secs(5));
        response = "OK";

    }

   
    stream.write_all(response.as_bytes()).unwrap();
    println!("A client has been finished {:?}", stream)
   

}
