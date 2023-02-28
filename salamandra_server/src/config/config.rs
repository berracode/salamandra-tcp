use std::{fs::File, io::Read};

use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// configuración del cliente
    #[serde(rename = "server")]
    pub server: Server,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    /// Socket addresses where this server listens.
    pub listen: Sock,

    /// server name
    pub name: Option<String>,

    pub base_route: Option<String>,

    pub buffer_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sock {
    pub ip: Option<String>,
    pub port: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        let mut file_content = String::new();
        let mut file = File::open("server.toml").unwrap();
        file.read_to_string(&mut file_content).unwrap();
        let mut config: Config = toml::from_str(&file_content).unwrap();
        let my_local_ip = local_ip().unwrap().to_string();
        config.server.listen.ip = Some(my_local_ip);

        println!("config {:?}", config.get_listener());
        config
    }

    pub fn get_listener(&self) -> String {
        let client = &self.server;
        let listen = &client.listen;
        let local_ip_address = listen.ip.clone();
        let local_port = listen.port.clone();

        let mut local_ip_address = local_ip_address.unwrap();
        let local_port = local_port.unwrap();
        local_ip_address.push_str(":");
        local_ip_address.push_str(&local_port);

        local_ip_address.clone()
    }

    pub fn get_ip(&self) -> String {
        let server = &self.server;
        let ip = server.listen.ip.clone();
        ip.unwrap()
    }

    pub fn get_port(&self) -> u16 {
        let server = &self.server;
        let port = server.listen.port.clone();
        let port = port.unwrap();
        port.parse::<u16>().unwrap()
    }
}
