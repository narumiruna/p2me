use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    process::{Command, Output},
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};

use serde_json::Value;

pub enum Reply {
    Json(u16, Value),
    InvalidJson,
    Redirect,
    Disconnect,
}

pub struct Request {
    pub line: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    fn read(stream: &mut TcpStream) -> Option<Self> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            return None;
        }
        let mut headers = HashMap::new();
        loop {
            let mut header = String::new();
            assert_ne!(reader.read_line(&mut header).unwrap(), 0);
            if header == "\r\n" {
                break;
            }
            let (name, value) = header.split_once(':').unwrap();
            headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
        }
        let length = headers
            .get("content-length")
            .map_or(0, |size| size.parse().unwrap());
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        Some(Self {
            line: line.trim_end().to_owned(),
            headers,
            body,
        })
    }
}

pub fn invoke(mut command: Command, replies: Vec<Reply>) -> (Output, Vec<Request>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    command.env(super::TEST_PORT, port.to_string());
    thread::scope(|scope| {
        let (done, stop) = mpsc::channel();
        let server = scope.spawn(move || {
            let mut requests = Vec::new();
            let mut replies = replies.into_iter();
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let timeout = Some(Duration::from_secs(3));
                        stream.set_read_timeout(timeout).unwrap();
                        stream.set_write_timeout(timeout).unwrap();
                        if let Some(request) = Request::read(&mut stream) {
                            requests.push(request);
                            let reply = replies.next().unwrap_or(Reply::Json(500, Value::Null));
                            respond(&mut stream, reply, port);
                        }
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        if !matches!(stop.try_recv(), Err(TryRecvError::Empty)) {
                            return requests;
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => panic!("test server accept failed: {error}"),
                }
            }
        });
        let output = command.output();
        let _ = done.send(());
        (output.unwrap(), server.join().unwrap())
    })
}

fn respond(stream: &mut TcpStream, reply: Reply, port: u16) {
    let (status, headers, body) = match reply {
        Reply::Json(204, _) => (204, String::new(), String::new()),
        Reply::Json(code, body) => (code, String::new(), body.to_string()),
        Reply::InvalidJson => (200, String::new(), "not JSON".into()),
        Reply::Redirect => (
            307,
            format!("Location: http://localhost:{port}/redirected\r\n"),
            String::new(),
        ),
        Reply::Disconnect => return,
    };
    write!(
        stream,
        "HTTP/1.1 {status} Fixture\r\nConnection: close\r\nContent-Type: application/json\r\n{headers}Content-Length: {}\r\n\r\n{body}",
        body.len(),
    )
    .unwrap();
}
