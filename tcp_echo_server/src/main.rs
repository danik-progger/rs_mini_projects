use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn handle_client(mut stream: TcpStream) {
    let peer_addr = stream
        .peer_addr()
        .map_or_else(|_| "unknown".to_string(), |addr| addr.to_string());
    println!("Handling connection from {}", peer_addr);
    let mut buff = [0; 1024];

    loop {
        match stream.read(&mut buff) {
            Ok(n) => {
                if n == 0 {
                    println!("Client {} closed connection", peer_addr);
                    break;
                }

                if let Err(e) = stream.write_all(&buff[0..n]) {
                    eprintln!("Write error to client {}: {}", peer_addr, e);
                    break;
                }
            }
            Err(e) if io::ErrorKind::Interrupted == e.kind() => continue,
            Err(e) if io::ErrorKind::ConnectionReset == e.kind() => {
                println!("Client {} reset connection", peer_addr);
                continue;
            }
            Err(e) => {
                eprintln!("Read error: {} from client: {}", e, peer_addr);
            }
        }
        break;
    }

    println!("Closed connection for user {}", peer_addr);
}

fn main() {
    println!("Hello, world!");
    let addr = "127.0.0.1:9090".to_string();

    let server = TcpListener::bind(&addr).expect("Failed to bind");

    println!("🟢 Server started at {}", addr);

    for res in server.incoming() {
        match res {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to adopt connection {}", e);
            }
        }
    }
}
