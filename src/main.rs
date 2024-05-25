use flate2::write::GzEncoder;
use flate2::Compression;
use nom::AsBytes;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::TcpListener;

fn convert_to_vector(content: String) -> Vec<String> {
    let mut vec_string: Vec<String> = Vec::new();
    let mut i = 0;
    for j in 0..content.len() {
        if content.chars().nth(j).unwrap() == '\n' {
            if i != j - 1 {
                //no empty string
                vec_string.push(content[i..j - 1].to_string()); // Ignore \r as well
            }
            i = j + 1;
        }
    }
    vec_string.push(content[i..].to_string());
    println!("vec_string: {:?}", vec_string);
    vec_string
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let _handle = std::thread::spawn(move || {
                    let mut request_buffer = [0; 512];
                    let request_buffer_size =
                        stream.read(&mut request_buffer).expect("HTTP Request");

                    let request_string =
                        String::from_utf8_lossy(&request_buffer[..request_buffer_size]).to_string();
                    println!("request_string: {:?}", request_string);
                    let lines = convert_to_vector(request_string);

                    let req_line = lines.first().unwrap();

                    let target = req_line.split_whitespace().nth(1).unwrap();

                    if target == "/" {
                        stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                    } else if target.starts_with("/echo/") {
                        let has_accept_encoding = lines
                            .iter()
                            .any(|line| line.starts_with("Accept-Encoding:"));
                        let body = target.split("/").last().expect("Cannot parse currently");

                        if !has_accept_encoding {
                            if body != "/" {
                                stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).as_bytes()).unwrap();
                            }
                        } else {
                            let encoding_types = lines
                                .iter()
                                .find(|line| line.to_lowercase().starts_with("accept-encoding:"))
                                .unwrap()
                                .strip_prefix("Accept-Encoding: ")
                                .unwrap()
                                .split(", ")
                                .collect::<Vec<&str>>();

                            println!("encoding_types: {:?}", encoding_types);

                            if encoding_types
                                .iter()
                                .any(|line| line.to_lowercase().eq("gzip"))
                            {
                                let mut encoder =
                                    GzEncoder::new(Vec::new(), Compression::default());
                                encoder
                                    .write_all(body.as_bytes())
                                    .expect("Failed to write to encoder");
                                let compressed_body =
                                    encoder.finish().expect("Failed to finish encoding");
                                let response = [
                                    "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: text/plain\r\nContent-Length:".as_bytes(),
                                    compressed_body.len().to_string().as_bytes(),
                                    "\r\n\r\n".as_bytes(),
                                    compressed_body.as_bytes()
                                    ].concat();
                                stream.write(&response).unwrap();
                            } else {
                                stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).as_bytes()).unwrap();
                            }
                        }
                    } else if target.starts_with("/user-agent") {
                        for i in 1..lines.len() {
                            if lines[i].to_lowercase().starts_with("user-agent") {
                                let header_val = lines[i].split_whitespace().nth(1).unwrap();
                                let fmt  = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", header_val.len(), header_val);
                                stream.write(fmt.as_bytes()).unwrap();
                                break;
                            }
                        }
                    } else if target.starts_with("/files/") {
                        let type_of_request = req_line.split_whitespace().nth(0).unwrap();

                        let env = env::args().collect::<Vec<String>>();
                        let dirname = env.get(2).expect("No directory given").clone();
                        let filename = target.strip_prefix("/files/").expect("Invalid filename");
                        let filepath_string = format!("{}{}", dirname, filename);
                        let filepath = std::path::Path::new(filepath_string.as_str());

                        if type_of_request == "POST" {
                            let content = lines.last().unwrap().as_bytes();
                            println!("content: {:?}", content);
                            println!("filepath: {:?}", filepath);

                            let mut file = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(&filepath)
                                .expect("Cannot open file");

                            file.write_all(content).unwrap();

                            println!("file created");
                            stream
                                .write("HTTP/1.1 201 Created\r\n\r\n".as_bytes())
                                .unwrap();
                        } else if type_of_request == "GET" {
                            let file = std::fs::read(filepath);
                            if let Ok(file) = file {
                                let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n", file.len(), String::from_utf8(file).expect("file content"));
                                stream.write(resp.as_bytes()).unwrap();
                            } else {
                                stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                            }
                        }
                    } else {
                        stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
