use encoding::{all::ASCII, EncoderTrap, Encoding};
use std::str;
use std::{
    error,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    pin::Pin,
};

use crate::config::config::Config;

#[derive(Debug)]
pub struct Connection {
    pub config: Config,
    pub reader: BufReader<&'static TcpStream>,
    pub writer: BufWriter<&'static TcpStream>,
    pub stream: Pin<Box<TcpStream>>,
}

impl Connection {
    pub fn new(stream: TcpStream, config: Config) -> Self {
        let buffer_size = config.server.buffer_size.into();
        //let reader = BufReader::with_capacity(buffer_size, stream.try_clone()?);
        //let writer = BufWriter::with_capacity(buffer_size, stream.try_clone()?);

        let pin = Box::pin(stream);
        unsafe {
            Self {
                config,
                reader: BufReader::with_capacity(buffer_size, std::mem::transmute(&*pin)),
                writer: BufWriter::with_capacity(buffer_size, std::mem::transmute(&*pin)),
                stream: pin,
            }
        }
    }

    pub fn send_message(&mut self, message: &str) {
        println!("message to server: {}", message);
        let ack = encode_message(message).unwrap();
        //send ack
        self.writer.write_all(&ack).unwrap();
        self.writer.flush().unwrap();
        println!("[client Enviando] ACKS");
    }

    pub fn read_message(&mut self) -> Result<&[u8], io::Error> {
        let buf_vec = self.reader.fill_buf();
        buf_vec
    }

    pub fn base_route(&self) -> Option<String> {
        let client = self.config.server.clone();
        let route = client.base_route;
        route
    }
}

pub fn decode_message(buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message
        .chars()
        .filter(|message_byte| message_byte.is_ascii_graphic() == true)
        .collect();

    clean_message
}

pub fn encode_message(cmd: &str) -> Result<Vec<u8>, Box<dyn error::Error + Send + Sync>> {
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII
        .encode(&message_str, EncoderTrap::Strict)
        .map_err(|x| x.into_owned())?;
    message_bytes.push('\r' as u8);
    println!("mensaje codificado: {:?}", message_bytes);

    Ok(message_bytes)
}
