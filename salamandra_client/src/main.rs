use std::{net::{TcpListener}, thread, sync::Arc};
use salamandra_client::{config::config::Config, tcp::tcp::process_connection};


fn main() {
    println!("Hello, world client!");
    start_server();

}

fn start_server(){

    let config= Config::new();

    let listener = TcpListener::bind(config.get_listener()).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {

        let config = config.clone();
        thread::spawn(move || {
            let stream = stream.unwrap();
            process_connection(stream, config);
        });
    }
}

