use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use hell0::pool::thread_pool::ThreadPool;

fn main() {
    const MAX_WORKERS: u32 = 5;
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to address");
    let pool = ThreadPool::build(MAX_WORKERS).unwrap(); // if no threadpools then let it crash

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            },
            Err(e) => {
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Failed to read line: {}", e);
            return;
        }
        None => {
            eprintln!("No lines found in request");
            return;
        }
    };

    let (status_line, source_html_file_name) = match &request[..]  {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "err.html")
    };
    let contents = match fs::read_to_string(source_html_file_name) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Failed to read HTML file: {}", e);
            stream.write_all("HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n".as_bytes()).unwrap();
            return;
        }
    };
    let content_length: usize = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{contents}");
    
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response: {}", e);
    }
}
