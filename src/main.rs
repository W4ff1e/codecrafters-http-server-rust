use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    println!("Client connected: {}", stream.peer_addr().unwrap());

    let mut buf = [0; 1024];
    match stream.read(&mut buf) {
        Ok(0) => {
            println!("Client disconnected");
            return;
        }
        Ok(_) => {
            let request_str = String::from_utf8_lossy(&buf);
            if request_str.contains("GET / HTTP/1.1") {
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            } else {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
        Err(err) => {
            eprintln!("Error reading from socket: {}", err);
        }
    }
}

fn main() {
    let address = "127.0.0.1:4221";

    let listener = TcpListener::bind(address).expect("Failed to bind to address");

    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
}
