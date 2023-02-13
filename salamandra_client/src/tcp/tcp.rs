use std::{net::TcpStream, io::{BufWriter, Write, Read, BufReader, BufRead, self}, fs::File, error, fmt::Error, pin::Pin, time::Duration, thread};
use std::str;
use encoding::{EncoderTrap, all::ASCII, Encoding};

use crate::{config::config::Config, file::file::FileManager};

fn my_decode_message(mut buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message.chars().filter(|message_byte|{
        message_byte.is_numeric() == true 
    }).collect();

    println!("clean {}",clean_message);
    clean_message
}

pub fn process_connection(mut stream: TcpStream, config: Config){
    println!("una nueva conexión desde {:?}", stream);

    let mut connection = Connection::new(stream, config.clone());

    let mut buf_vec: Vec<u8> = connection.read_message().unwrap().to_vec();//buf_reader.fill_buf().unwrap().to_vec(); //8192 bytes buffer
    connection.reader.consume(buf_vec.len());
    println!("buf_vec {:?}", buf_vec.len());

    // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
    // buf_reader.consume(buf_vec.len());

    let file_size = my_decode_message(&mut buf_vec);
    println!("file_size {:?}", file_size);

    connection.send_message();

    let mut full_path = connection.get_route();

    let mut fullname = String::from(full_path.unwrap());
    fullname.push_str(&"principal.exe".to_string());

    let mut file_manager = FileManager::new(fullname, config.clone()).unwrap();


    //receive file itself (write to file)
    let mut remaining_data = file_size.parse::<i32>().unwrap();
    while remaining_data != 0 {

        //println!("remaining_data: {:?} >= connection.config.client.buffer_size: {:?} ", remaining_data, connection.config.client.buffer_size);

        if remaining_data >= connection.config.client.buffer_size as i32 {
                //let slab = stream.read(&mut buf);
            //let mut slab= buf_reader.fill_buf(); //8192 bytes buffer
            let mut slab = connection.read_message();

            match slab {
                Ok(n) => {
                    let mut slab: Vec<u8> = slab.unwrap().to_vec();
                    // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
                    connection.reader.consume(slab.len());
                    let n = slab.len();

                    //println!("datos recibidos {:?}", slab.len());

                    file_manager.write_file(slab);

                    //file_buffer.write(&mut slab).unwrap();
                    //file_buffer.flush().unwrap();
                    remaining_data = remaining_data - n as i32;
                    //println!("wrote {} bytes to file | remaining_data: {}", n, remaining_data);
                }
                _ => {}
            }
        } else {
            let array_limit = (remaining_data as i32) - 1;
            //let slab = stream.read(&mut buf);
            //let mut slab= buf_reader.fill_buf(); //8192 bytes buffer
            let mut slab= connection.read_message(); //8192 bytes buffer

            match slab {
                Ok(_) => {
                    //let mut r_slice = &buf[0..(array_limit as usize + 1)]; //fixes underreading
                    let mut slab: Vec<u8> = slab.unwrap().to_vec();
                    println!("============datos recibidos en else {:?}", slab.len());

                    println!("capacity: {:?}",file_manager.writer.capacity());
                    file_manager.update_capacity(remaining_data.try_into().unwrap());
                    println!("capacity: {:?}",file_manager.writer.capacity());

                    //caused by not using
                    //subprocess call on 
                    //the server
                    //let r = r_slice.to_vec();
                    file_manager.write_file(slab);

                    //file_buffer.write(&mut r_slice).unwrap();
                    //file_buffer.flush().unwrap();
                    println!("=========wrote {} bytes to file (small)", remaining_data as i32);
                    remaining_data = 0;
                }
                _ => {}
            }
        }
    }








    println!("termina cliente {:?}", connection.stream)

}


fn encode_message(cmd: &str) -> Result <Vec<u8>, Box<dyn error::Error + Send + Sync>> {
    //println!("{:?}", cmd);
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII.encode(&message_str, EncoderTrap::Strict).map_err(|x| x.into_owned())?;
    message_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_bytes)
}


#[derive(Debug)]
pub struct Connection{
    pub config: Config,
    pub reader: BufReader<&'static TcpStream>,
    pub writer: BufWriter<&'static TcpStream>,
    pub stream: Pin<Box<TcpStream>>,


}

impl Connection {

    pub fn new(stream:  TcpStream, config: Config) -> Self{
        let buffer_size = config.client.buffer_size.into();
        //let reader = BufReader::with_capacity(buffer_size, stream.try_clone()?);
        //let writer = BufWriter::with_capacity(buffer_size, stream.try_clone()?);

        let pin = Box::pin(stream);
        unsafe {
            Self{
                config,
                reader: BufReader::with_capacity(buffer_size,std::mem::transmute(&*pin)),
                writer: BufWriter::with_capacity(buffer_size,std::mem::transmute(&*pin)),
                stream: pin,
            }
        }


    }



    pub fn send_message(&mut self) {
          //send ack
        let ack = encode_message("ACK").unwrap();
        //send ack
        self.writer.write_all(&ack).unwrap();
        self.writer.flush().unwrap();
        println!("[client Enviando] ACKS");

        
    }

    pub fn read_message(&mut self) -> Result<&[u8], io::Error>{

        let buf_vec = self.reader.fill_buf();
        buf_vec
    }

    pub fn get_route(&self) -> Option<String> {
        let client = self.config.client.clone();
        let route = client.route_down;
        route
    }




    
}