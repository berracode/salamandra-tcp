use std::{net::TcpStream, io::{BufWriter, Write, Read}, fs::File, error};
use std::str;
use encoding::{EncoderTrap, all::ASCII, Encoding};

use crate::config::config::Config;

const BUFFERSIZE: usize = 8192;


pub fn process_connection(mut stream: TcpStream, config: Config){
    println!("una nueva conexión desde {:?}", stream);
    //let mut buf_reader = BufReader::new(&stream);
    //thread::sleep(Duration::from_secs(10));

    let mut buf = [0u8; 8192]; //8192 bytes buffer
    //send ack
    let ack = encode_message("ACK").unwrap();

    //read  file size
    stream.read(&mut buf).unwrap();
    let file_size = decode_message_size(&mut buf);
    println!("file_size {:?}", file_size);

    //send ack
    stream.write_all(&ack).unwrap();

    let mut fullname = String::from("./src/shared/");
    fullname.push_str(&"principal.exe".to_string());

    let mut file_buffer = BufWriter::new(File::create(fullname).unwrap());

     //receive file itself (write to file)
     let mut remaining_data = file_size.parse::<i32>().unwrap();
     while remaining_data != 0 {
         if remaining_data >= BUFFERSIZE as i32
         {
             let slab = stream.read(&mut buf);
             match slab {
                 Ok(n) => {
                    file_buffer.write(&mut buf).unwrap();
                    file_buffer.flush().unwrap();
                    remaining_data = remaining_data - n as i32;
                    println!("wrote {} bytes to file | remaining_data: {}", n, remaining_data);
                 }
                 _ => {}
             }
         } else {
             let array_limit = (remaining_data as i32) - 1;
             let slab = stream.read(&mut buf);
             match slab {
                 Ok(_) => {
                     let mut r_slice = &buf[0..(array_limit as usize + 1)]; //fixes underreading
                     //caused by not using
                     //subprocess call on 
                     //the server
                     file_buffer.write(&mut r_slice).unwrap();
                     file_buffer.flush().unwrap();
                     println!("wrote {} bytes to file (small)", remaining_data as i32);
                     remaining_data = 0;
                 }
                 _ => {}
             }
         }
     }







   /*  let mut final_line = String::new();
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

    stream.write_all(response.as_bytes()).unwrap();*/
    println!("termina cliente {:?}", stream)

}


fn encode_message(cmd: &str) -> Result <Vec<u8>, Box<dyn error::Error + Send + Sync>> {
    //println!("{:?}", cmd);
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII.encode(&message_str, EncoderTrap::Strict).map_err(|x| x.into_owned())?;
    message_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_bytes)
}

fn decode_message_size(mut ack_buf: &mut [u8]) -> String {
    let msg_len_slice: &str = str::from_utf8(&mut ack_buf).unwrap();
    let mut msg_len_str = msg_len_slice.to_string();
    let mut numeric_chars = 0;
    for c in msg_len_str.chars() {
        if c.is_numeric() == true {
            numeric_chars = numeric_chars + 1;
        }
    }
    //shrink:
    msg_len_str.truncate(numeric_chars);
    msg_len_str
}