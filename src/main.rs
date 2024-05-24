use std::env;
use std::io::{BufRead, BufReader, Write};
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
            Ok(mut _stream) => {
                println!("accepted new connection");
                let request: &mut Vec<u8> = &mut Vec::new();
                let mut buf_reader = BufReader::new(&mut _stream);

                buf_reader.read_until('\0' as u8, request).unwrap();
                let request_immutable = &*request.clone();
                let request_string = String::from_utf8(request_immutable.to_vec()).unwrap();

                let lines = convert_to_vector(request_string);

                let req_line = lines.first().unwrap();

                let target = req_line.split_whitespace().nth(1).unwrap();

                if target == "/" {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else if target.starts_with("/echo/") {
                    let body = target.split("/").last().expect("Cannot parse currently");
                    if body != "/" {
                        _stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).as_bytes()).unwrap();
                    }
                } else if target.starts_with("/user-agent") {
                    for i in 1..lines.len() {
                        if lines[i].starts_with("User-Agent") {
                            let header_val = lines[i].split_whitespace().nth(1).unwrap();
                            let fmt  = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", header_val.len(), header_val);
                            _stream.write(fmt.as_bytes()).unwrap();
                            break;
                        }
                    }
                } else if target.starts_with("/files/") {
                    let type_of_request = req_line.split_whitespace().nth(0).unwrap();

                    let env = env::args().collect::<Vec<String>>();
                    let dirname = env.get(2).expect("No directory given").clone();
                    let filename = target.strip_prefix("/files/").expect("Invalid filename");
                    let filepath = format!("{}{}", &dirname, filename);

                    if type_of_request == "POST" {
                        let content = lines.last().unwrap().as_bytes();
                        println!("content: {:?}", content);
                        println!("filepath: {:?}", filepath);
                        if let Ok(_) = std::fs::write(filepath, content) {
                            let resp = format!("HTTP/1.1 200 OK\r\n\r\n");
                            _stream.write(resp.as_bytes()).unwrap();
                        } else {
                            println!("Couldn't write in file");
                            _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                        }
                        // std::fs::write(filepath, content).expect("Couldn' write in file");

                        // let resp = format!("HTTP/1.1 201 Created\r\n\r\n");
                        // _stream.write(resp.as_bytes()).unwrap();
                    } else if type_of_request == "GET" {
                        let file = std::fs::read(filepath);
                        if let Ok(file) = file {
                            let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n", file.len(), String::from_utf8(file).expect("file content"));
                            _stream.write(resp.as_bytes()).unwrap();
                        } else {
                            _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                        }
                    }
                } else {
                    _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
