use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

fn convert_to_vector(content: String) -> Vec<String> {
    let mut vec_string: Vec<String> = Vec::new();
    // for mut i in 0..content.len() {
    //     // split by \n
    //     let mut line = String::new();
    //     while i < content.len() && content.chars().nth(i).unwrap() != '\n' {
    //         line.push(content.chars().nth(i).unwrap());
    //         i += 1;
    //     }
    //     println!("line_inside_function: {}", line);
    //     vec_string.push(line);
    // }
    //
    let mut i = 0;
    for j in 0..content.len() {
        if content.chars().nth(j).unwrap() == '\n' {
            vec_string.push(content[i..j - 1].to_string()); // Ignore \r as well
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
                let content: &mut Vec<u8> = &mut Vec::new();
                let mut buf_reader = BufReader::new(&mut _stream);

                buf_reader.read_until('\0' as u8, content).unwrap();
                let content_immutable = &*content.clone();
                let stringistring = String::from_utf8(content_immutable.to_vec()).unwrap();
                println!("content: {}", stringistring);

                let lines = stringistring
                    .lines()
                    .take_while(|line| !line.is_empty())
                    .collect::<Vec<&str>>();
                println!("lines: {:?}", lines);
                let lines = convert_to_vector(stringistring);

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
                    let mut dirname = env.get(2).expect("No directory given").clone();
                    let filename = target.split("/").last().expect("Invalid filename");
                    dirname.push_str(filename);

                    if type_of_request == "POST" {
                        // let content = req_line
                        //     .split_whitespace()
                        //     .last()
                        //     .expect("No content given");
                        // let content = lines.last().unwrap();
                        let file = std::fs::write(dirname, content);

                        println!("array {:?}", lines);
                        // println!("content: {}", content);
                        if let Ok(_) = file {
                            // let resp = format!("HTTP/1.1 201 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n", content.len(), String::from_utf8(file).expect("file content"));
                            let resp = format!("HTTP/1.1 201 Created\r\n\r\n");
                            _stream.write(resp.as_bytes()).unwrap();
                        } else {
                            _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                        }
                    } else if type_of_request == "GET" {
                        let file = std::fs::read(dirname);
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
