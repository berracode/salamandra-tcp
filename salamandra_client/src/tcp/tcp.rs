use std::{net::TcpStream, io::{Write, Seek, BufRead}, 
fs::File, time::Duration, thread, cmp::min};
use std::str;
use serde_json::json;


use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use tokio::runtime;

use crate::{config::config::Config, tcp::connection::{Connection, self}};

/// El cliente trata de enviarle al servidor sus datos para registrarse en la
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
                let response_message = connection::decode_message(&mut buf_vec);

                if response_message.eq_ignore_ascii_case("OK") {
                    break;
                }

            },
            Err(error) => {
                eprintln!("Error: {error}");
            },
        }
        thread::sleep(Duration::from_secs(5));
        println!("Reintantando...");
    }
}

async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
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

pub fn process_connection(stream: TcpStream, config: Config) {
    println!("una nueva conexión desde {:?}", stream);

    let mut connection = Connection::new(stream, config.clone());



    //tratar de registrarse en el server

    let mut buf_vec: Vec<u8> = connection.read_message().unwrap().to_vec();//buf_reader.fill_buf().unwrap().to_vec(); //8192 bytes buffer
    connection.reader.consume(buf_vec.len());
    println!("buf_vec {:?}", buf_vec.len());
    let responsa_message = connection::decode_message(&mut buf_vec);
    println!("response_message: {}", responsa_message);

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .worker_threads(2)
        .max_blocking_threads(2)
        .build().unwrap();

    println!("con runtime");
    rt.block_on( async move  {
        let client = reqwest::Client::new();
        download_file(&client, responsa_message.as_str(), "./shared/android.tar.gz").await.unwrap();
    });

    //avisar al server

    connection.send_message("downloaded");


    println!("termina cliente {:?}", connection.stream)

}




/// Con este test simulamos que el servidor le envia al cliente la URL para descargar archivo.
#[test]
fn connection_broadcats() {
    let stream = TcpStream::connect("192.168.1.161:9090").unwrap();
    let config= Config::new();
    let mut conn_server = Connection::new(stream, config);

    let url = "http://192.168.1.161:8080/shared/android.tar.gz";
    conn_server.send_message(url);

    let mut response_client = conn_server.read_message().unwrap().to_vec();
    conn_server.reader.consume(response_client.len());
    let response_message = connection::decode_message(&mut response_client);

    println!("client say: {}", response_message);

    assert_eq!(String::from("downloaded"), response_message);
}
