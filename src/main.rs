use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    const RESPONSE_200: &str = "HTTP/1.1 200 OK\r\n\r\n";
    const RESPONSE_404: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let path = extract_path(&mut request_line);

                match path {
                    "/" => stream.write_all(RESPONSE_200.as_bytes()).unwrap(),

                    _ => stream.write_all(RESPONSE_404.as_bytes()).unwrap(),
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn extract_path(request_line: &str) -> &str {
    let parts = request_line.split(" ").collect::<Vec<&str>>();
    parts[1]
}
