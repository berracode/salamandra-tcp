use std::{io::{BufReader, BufWriter, self, Write}, fs::File};

use crate::config::config::Config;


pub struct FileManager {
    pub reader: BufReader<File>,
    pub writer: BufWriter<File>,
    pub config: Config,
    pub file: File
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
            file
        })
    }

    pub fn write_file(&mut self, mut buf: Vec<u8>) {

        //println!("datos a escribir write {:?}", buf.len());

        self.writer.write(&mut buf).unwrap();
        self.writer.flush().unwrap();

    }

    pub fn update_capacity(&mut self, buffer_size: usize) {
        self.writer = BufWriter::with_capacity(buffer_size, self.file.try_clone().unwrap());

    }
}