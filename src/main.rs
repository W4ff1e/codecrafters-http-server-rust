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

                let mut method = "";
                let mut url = "";
                if request_parts.len() >= 2 {
                    method = request_parts[0];
                    url = request_parts[1];
                }

                println!("Method: {}, Requested URL: {}", method, url);

                if method == "GET" {
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
                } else if method == "POST" {
                    if url.starts_with("/files/") {
                        let content_length = extract_content_length(&request_str);
                        let body_start = request_str.find("\r\n\r\n").unwrap_or(0) + 4;
                        let body = &request_str[body_start..];

                        println!("POST body: {}", body);

                        let file_path = format!("{}{}", directory, &url[7..]);
                        println!("File Path: {}", file_path);

                        if let Err(err) = save_file(&file_path, body, content_length) {
                            eprintln!("Error saving file: {}", err);
                            let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
                            stream.write_all(response.as_bytes()).unwrap();
                            return;
                        }

                        println!("File saved to: {}", file_path);

                        let response = "HTTP/1.1 201 Created\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    } else {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else {
                    let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
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

fn extract_content_length(request: &str) -> usize {
    let mut content_length = 0;
    for line in request.lines() {
        if line.starts_with("Content-Length:") {
            if let Some(len) = line.split(":").nth(1) {
                content_length = len.trim().parse().unwrap_or(0);
            }
        }
    }
    content_length
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

fn save_file(file_path: &str, contents: &str, content_length: usize) -> Result<(), std::io::Error> {
    let trimmed_contents = contents.trim_end_matches('\0');
    let truncated_contents = &trimmed_contents[..content_length];
    fs::write(file_path, truncated_contents)
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
