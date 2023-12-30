use rust_http_server::{
    mime_types::MimeType, request::Request, response::Response,
};
use std::{
    error::Error, io::BufRead, net::TcpListener, num::IntErrorKind, path::Path, sync::RwLock,
    thread,
};

enum IndexStyle {
    NotFound,
    IndexDirectory,
    IndexFile(String),
}

static mut INDEX_STYLE: RwLock<IndexStyle> = RwLock::new(IndexStyle::IndexDirectory);

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);

    let mut port = 8080;
    let mut external = false;

    while let Some(arg) = args.next() {
        if arg == "-p" || arg == "--port" {
            port = match args.next().map(|arg| arg.parse::<u16>()) {
                Some(Ok(port)) => port,
                Some(Err(ref err)) if err.kind() == &IntErrorKind::PosOverflow => {
                    panic!("Port has to be in range: 0 < port < {}", u16::MAX);
                }
                _ => panic!(),
            };
        } else if arg == "-o" || arg == "--open" {
            external = true;
        } else if arg == "-i" || arg == "--index" || arg == "--index-style" {
            let is = args
                .next()
                .expect("index style missing: (dir | [filename] | none)");

            *(unsafe { INDEX_STYLE.write() }).expect("failed to get write lock") = {
                match is.to_lowercase().as_str() {
                    "dir" => IndexStyle::IndexDirectory,
                    "none" => IndexStyle::NotFound,
                    _ => IndexStyle::IndexFile(is),
                }
            }
        } else {
            eprintln!("argument not recognized: {arg:?}")
        }
    }

    let port_range_end = port.checked_add(9).unwrap_or(u16::MAX);
    thread::spawn(move || {
        let listener = match TcpListener::bind(
            &(port..=port_range_end)
                .map(|port| {
                    if external {
                        std::net::SocketAddr::from(([0, 0, 0, 0], port))
                    } else {
                        std::net::SocketAddr::from(([127, 0, 0, 1], port))
                    }
                })
                .collect::<Vec<_>>()[..],
        ) {
            Ok(listener) => {
                let local_addr = listener.local_addr().unwrap();

                let actual_host = local_addr.ip();
                let actual_port = local_addr.port();

                if actual_port != port {
                    println!("Could not bind to {port}, using {actual_port} instead.");
                }

                println!("Listening on {actual_host}:{actual_port}...");

                listener
            }
            Err(err) => {
                panic!("{err}");
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        let req: Request = stream.try_into().expect("Failed to parse request");
                        handler(&req).respond_to(req).expect("Failed to respond");
                    });
                }
                Err(e) => {
                    println!("Failed to read stream, {e:?}");
                }
            }
        }
    });

    let mut stdin = std::io::stdin().lock();

    loop {
        let mut command = String::new();
        stdin.read_line(&mut command).unwrap();
        let command = &command[..command.len() - 1];

        match command {
            "exit" | "quit" | "q" => break,
            _ => println!(">> [info] q to quit"),
        }
    }

    Ok(())
}

const DIR: &str = "./public/";
fn handler(req: &Request) -> Response {
    assert!(req.pathname.starts_with('/'));

    let path = Path::new(DIR).join(String::from(".") + req.pathname.clone().as_str());

    if !path.exists() {
        Response::builder().status(404).build()
    } else if path.is_file() {
        let mime_type = MimeType::get_for_path(&req.pathname);
        match std::fs::read(path).map(String::from_utf8) {
            Ok(Ok(file)) => Response::builder()
                .status(200)
                .header("Content-Type", mime_type)
                .body(file)
                .build(),
            _ => Response::builder().status(500).build(),
        }
    } else if path.is_dir() {
        match *(unsafe { INDEX_STYLE.read() }).expect("failed to get index style") {
            IndexStyle::IndexDirectory => match std::fs::read_dir(path) {
                Ok(files) => {
                    let files = files
                        .filter_map(|file| Some(file.ok()?.file_name().to_str()?.to_string()))
                        .fold(String::new(), |mut out, name| {
                            out += format!(r#"<li><a href="./{name}">{name}</a></li>"#).as_str();
                            out
                        });

                    Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .body(format!(
                            r#"<!DOCTYPE html>
<html lang="en" dir="ltr">
<head>
    <title>Index of {pathname}</title>
    <style>:root{{color-scheme:light dark;}}</style>
</head>
<body>
    <h1>Index of {pathname}</h1>
    <ul>{files}</ul>
</body>
</html>
"#,
                            pathname = req.pathname
                        ))
                        .build()
                }
                Err(..) => Response::builder().status(500).build(),
            },
            IndexStyle::NotFound => Response::builder().status(404).build(),
            IndexStyle::IndexFile(..) => {
                // TODO:
                Response::builder().status(500).build()
            }
        }
    } else {
        unreachable!()
    }
}
