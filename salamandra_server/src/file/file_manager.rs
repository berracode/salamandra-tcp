use std::{fs::{ OpenOptions}, io::Write, net::{SocketAddr, Ipv4Addr}};

const ENTER: &str = "\r\n";
const FILE_DATA: &str = "file_data";


/// Save a new client in file data
/// Those clients would be using for send download information
pub fn save_client(new_client: SocketAddr) -> u16 {

    let mut new_client = new_client.to_string();
    new_client.push_str(ENTER);
    let mut file_data = OpenOptions::new()
                            .append(true) 
                            .create(true) 
                            .open(FILE_DATA).unwrap();

    file_data.write_all(new_client.as_bytes()).unwrap();
    println!("Client has been written");
    1
}



#[test]
fn save_client_test() {
    let new_client = vec![
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 1, 1)), 8080),
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 2, 1)), 8080),
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 3, 1)), 8080)
    ];

    let mut response = 0;
    for client in new_client {
        response = save_client(client.to_owned());
        
    }

    assert_eq!(1, response);
}



