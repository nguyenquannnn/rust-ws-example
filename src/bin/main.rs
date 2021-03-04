use hello::ThreadPool;
use std::env;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
// use std::time::Duration;

struct Setting {
    ws_port: String,
    root_dir: String,
    default_file: String,
}

impl Default for Setting {
    fn default() -> Self {
        Setting {
            ws_port: String::from("8082"),
            root_dir: String::from("./"),
            default_file: String::from("hello-world.html"),
        }
    }
}

fn main() {
    let mut setting = Arc::new(Setting::default());
    let port = match env::var("RUST_WS_PORT") {
        Ok(val) => val,
        Err(_e) => setting.ws_port.clone(),
    };
    let root_dir = match env::var("RUST_ROOT_DIR") {
        Ok(val) => val,
        Err(_e) => setting.root_dir.clone(),
    };
    *Arc::get_mut(&mut setting).unwrap() = Setting {
        ws_port: port,
        root_dir: root_dir,
        ..Setting::default()
    };
    let listener = TcpListener::bind(format!("127.0.0.1:{port}", port = setting.ws_port)).unwrap();
    let thread_pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let setting_ref = Arc::clone(&setting);
        thread_pool.execute(move || {
            println!("Connection establised!");
            handle_connection(stream, setting_ref);
        });
    }
}

// fn old_handle_connection(mut stream: TcpStream, setting: &Setting) {
//     let mut buffer = [0; 512];
//     stream.read(&mut buffer).unwrap();

//     let get = b"GET / HTTP/1.1\r\n";
//     let sleep = b"GET /sleep HTTP/1.1\r\n";
//     println!("------------------  REQUEST  ------------------");
//     println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
//     let response = if buffer.starts_with(get) {
//         format!(
//             "HTTP/1.1 200 OK \r\n\r\n{}",
//             fs::read_to_string("hello-world.html").unwrap()
//         )
//     } else if buffer.starts_with(sleep) {
//         std::thread::sleep(Duration::from_secs(5));
//         format!(
//             "HTTP/1.1 200 OK \r\n\r\n{}",
//             fs::read_to_string("hello-world.html").unwrap()
//         )
//     } else {
//         format!(
//             "{}{}",
//             "HTTP/1.1 4040 NOT FOUND\r\n\r\n",
//             fs::read_to_string("404.html").unwrap()
//         )
//     };
//     send_response(stream, response);
// }

fn handle_connection(mut stream: TcpStream, setting: Arc<Setting>) {
    let mut buffer = [0; 1028];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("------------------  REQUEST  ------------------");
    println!("Request:\r\n{}", request);
    let response = handle_request(parse_request(request.to_string()).unwrap(), setting);
    // let response = format!("HTTP/1.1 200 OK \r\n\r\n{}", String::from("Hello, World!"));
    send_response(stream, response);
}

#[derive(Debug)]
enum Method {
    GET,
    PUT,
    POST,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Request {
    method: Method,
    path: String,
}

fn parse_request(req: String) -> Option<Request> {
    let request = None;
    for (i, line) in req.lines().enumerate() {
        if i == 0 {
            let mut status_tokens = line.split_ascii_whitespace();
            let method = match status_tokens.next().unwrap() {
                "GET" => Method::GET,
                "PUT" => Method::PUT,
                "POST" => Method::POST,
                def => panic!("Unknown HTTP methods: {}", def),
            };
            let path = String::from(status_tokens.next().unwrap());
            let request = Some(Request { method, path });
            return request;
        }
    }
    request
}

fn handle_request(req: Request, setting: Arc<Setting>) -> String {
    let response;
    match req.method {
        Method::GET => {
            let mut path = PathBuf::new();
            path.push(&setting.root_dir);
            path.push(if req.path == "/" {
                &setting.default_file
            } else {
                &req.path[1..]
            });
            response = format!(
                "HTTP/1.1 200 OK \r\n\r\n{}",
                fs::read_to_string(path.to_str().unwrap()).unwrap()
            );
        }
        def => panic!("Unhandle METHOD: {}", def.to_string()),
    }
    response
}

fn send_response(mut stream: TcpStream, response: String) {
    println!("----------------- RESPONSE ------------------");
    println!("{}", response);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
