use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn handle_client(mut stream: TcpStream, directory: &str) {
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

                    if url.starts_with("/files/") {
                        let file_path = format!("{}{}", directory, &url[7..]);
                        println!("Requested File Path: {}", file_path);
                        if let Ok(contents) = read_file(&file_path) {
                            let content_length = contents.len();

                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", content_length, contents
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                        } else {
                            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                    } else if url == "/user-agent" {
                        let user_agent = extract_user_agent(&request_str);
                        println!("Extracted User Agent: {}", user_agent);
                        let content_length = user_agent.len();

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", content_length, user_agent
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    } else if url.starts_with("/echo/") {
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

fn extract_user_agent(request: &str) -> String {
    for line in request.lines() {
        if line.starts_with("User-Agent:") {
            return line.trim_start_matches("User-Agent: ").to_string();
        }
    }
    String::from("Unknown User-Agent")
}

fn read_file(file_path: &str) -> Result<String, std::io::Error> {
    if Path::new(&file_path).exists() {
        fs::read_to_string(file_path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() == 3 && args[1] == "--directory" {
        &args[2]
    } else {
        "./" // Default to the current working directory if --directory flag is not provided
    };

    let address = "127.0.0.1:4221";

    let listener = TcpListener::bind(address).expect("Failed to bind to address");

    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = directory.to_string();
                std::thread::spawn(move || {
                    handle_client(stream, &directory);
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
}
