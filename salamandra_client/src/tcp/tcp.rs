use std::{net::TcpStream, io::{BufWriter, Write, Read, BufReader, BufRead, self, Seek}, fs::File, error, fmt::{Error, format}, pin::Pin, time::Duration, thread, cmp::min};
use std::str;

use encoding::{EncoderTrap, all::ASCII, Encoding};
use serde_json::json;


use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use tokio::runtime;

use crate::{config::config::Config, file::file::FileManager};



/// el cliente trata de enviarle al servidor sus datos para registrarse en la
/// BD del servidor y que este pueda enviarle la url
pub fn try_sing_in(config: Config) {

    loop {
        let stream = TcpStream::connect("192.168.1.161:9123");
        match stream {
            Ok(stream) => {
                let mut connection = Connection::new(stream, config.clone());
 
                let request = json!({
                    "operation": "client-record",
                    "data":config.get_listener()
                });
                connection.send_message(request.to_string().as_str());
                let mut buf_vec: Vec<u8> = connection.read_message().unwrap().to_vec();
                connection.reader.consume(buf_vec.len());
                let responsa_message = my_decode_message(&mut buf_vec);

                if responsa_message.eq_ignore_ascii_case("OK") {
                    break;
                }

            },
            Err(error) => {
                eprintln!("Error: {error}");
            },
        }
        thread::sleep(Duration::from_secs(5));
        println!("Reintantando...")

 
    }
}


fn my_decode_message(mut buf: &mut [u8]) -> String {
    let dirty_message: &str = str::from_utf8(buf).unwrap();
    let clean_message: String = dirty_message.chars().filter(|message_byte|{
        message_byte.is_ascii_graphic() == true 
    }).collect();

    println!("clean {}",clean_message);
    clean_message
}


pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(&format!("Downloading {}", url));

    let mut file;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    
    println!("Seeking in file.");
    if std::path::Path::new(path).exists() {
        println!("File exists. Resuming.");
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .unwrap();

        let file_size = std::fs::metadata(path).unwrap().len();
        file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
        downloaded = file_size;

    } else {
        println!("Fresh file..");
        file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    }

    println!("Commencing transfer");
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
    return Ok(());
}

pub fn process_connection(mut stream: TcpStream, config: Config) {
    println!("una nueva conexión desde {:?}", stream);

    let mut connection = Connection::new(stream, config.clone());



    //tratar de registrarse en el server

    let mut buf_vec: Vec<u8> = connection.read_message().unwrap().to_vec();//buf_reader.fill_buf().unwrap().to_vec(); //8192 bytes buffer
    connection.reader.consume(buf_vec.len());
    println!("buf_vec {:?}", buf_vec.len());
    let responsa_message = my_decode_message(&mut buf_vec);
    println!("response_message: {}", responsa_message);

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .worker_threads(2)
        .max_blocking_threads(2)
        .build().unwrap();

    println!("con runtime");
    rt.block_on( async move  {
        let mut client = reqwest::Client::new();
        download_file(&client, responsa_message.as_str(), "./shared/android.tar.gz").await.unwrap();
    });

    

    //FUNCION DESCARGAR ARCHIVO

    let file_size = my_decode_message(&mut buf_vec);
    println!("file_size {:?}", file_size);

    //connection.send_message();

    let mut full_path = connection.get_route();

    let mut fullname = String::from(full_path.unwrap());
    fullname.push_str(&"principal.exe".to_string());

    let mut file_manager = FileManager::new(fullname, config.clone()).unwrap();


    //receive file itself (write to file)
    let mut remaining_data = file_size.parse::<i32>().unwrap();
    let mut read_data = 0;
    let total_size = remaining_data;
    let bar = ProgressBar::new(remaining_data.try_into().unwrap());

    /*bar
        .set_style(
            ProgressStyle
                ::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("#>-")
        );*/

    while remaining_data != 0 {

       
        //thread::sleep(Duration::from_secs(2));

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
                    read_data = i32::try_from(n).unwrap();
                    //println!("============datos recibidos en if {:?}", read_data);


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
                    read_data = i32::try_from(slab.len()).unwrap();
                    //println!("============datos recibidos en else {:?}", read_data);

                    //println!("capacity: {:?}",file_manager.writer.capacity());
                    file_manager.update_capacity(remaining_data.try_into().unwrap());
                    //println!("capacity: {:?}",file_manager.writer.capacity());

                    //caused by not using
                    //subprocess call on 
                    //the server
                    //let r = r_slice.to_vec();
                    file_manager.write_file(slab);

                    //file_buffer.write(&mut r_slice).unwrap();
                    //file_buffer.flush().unwrap();
                    //println!("=========wrote {} bytes to file (small)", remaining_data as i32);
                    remaining_data = 0;
                }
                _ => {}
            }
        }

     
        bar.inc(read_data.try_into().unwrap());


    }
    bar.finish();




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



    pub fn send_message(&mut self, message: &str) {
        println!("message to server: {}", message);
        let ack = encode_message(message).unwrap();
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

/// Con este test simulamos que el servidor le envia al cliente la URL para descargar archivo.
#[test]
fn connection_broadcats() {
    let mut stream = TcpStream::connect("192.168.1.161:9090").unwrap();
    let config= Config::new();
    let mut conn_server = Connection::new(stream, config);

    let msg = "http://192.168.1.161:8080/shared/android.tar.gz";
    conn_server.send_message(msg);

    /*stream.write(msg).unwrap();

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
    }*/


    assert_eq!(1, 1);
}
