use std::{io::{BufReader, BufWriter, self, Write}, fs::File};

use crate::config::config::Config;




pub struct FileManager {
    pub reader: BufReader<File>,
    pub writer: BufWriter<File>,
    pub config: Config
}

impl FileManager {
    
    pub fn new(full_name: String, config: Config) -> io::Result<Self>{
        let file = File::create(full_name)
        .unwrap_or_else(|error|{panic!("Error creando archivo: {}", error)});

        let buffer_size = config.client.buffer_size.into();
        let reader = BufReader::with_capacity(buffer_size, file.try_clone()?);
        let writer = BufWriter::with_capacity(buffer_size, file.try_clone()?);

        Ok(Self{
            reader,
            writer, 
            config,
        })
    }

    pub fn write_file(&mut self, mut buf: Vec<u8>) {
        let n = buf.len();

        println!("datos a escribir {:?}", buf.len());

        self.writer.write(&mut buf).unwrap();
        self.writer.flush().unwrap();

    }
}