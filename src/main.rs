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
            if let Some(request_line) = request_str.lines().next() {
                let request_parts: Vec<&str> = request_line.split_whitespace().collect();
                if request_parts.len() >= 2 && request_parts[0] == "GET" {
                    let url = request_parts[1];
                    println!("Requested URL: {}", url);

                    if url.starts_with("/echo/") {
                        // Extract the part after "/echo/"
                        let bodycontent = &url[6..];
                        println!("Extracted Content: {}", bodycontent);

                        let contentlength = bodycontent.len();
                        println!("Content Length: {}", contentlength);

                        if contentlength == 0 {
                            println!("Body content is empty!");
                        }

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", contentlength, bodycontent
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    } else if url == "/" {
                        let response = "HTTP/1.1 200 OK\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    } else {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
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
