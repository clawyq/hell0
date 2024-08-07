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
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::build(MAX_WORKERS).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request: String = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, source_html_file_name) = match &request[..]  {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "err.html")
    };
    let contents: String = fs::read_to_string(source_html_file_name).unwrap();
    let content_length: usize = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{contents}");
    
    stream.write_all(response.as_bytes()).unwrap();
}
