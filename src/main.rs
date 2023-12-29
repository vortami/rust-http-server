mod macros;

use rust_http_server::{
    common::Method,
    request::{Body, Request, RequestFromTcpStreamError},
};

use std::{
    error::Error,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    num::IntErrorKind,
    thread,
    time::SystemTime,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut port = 8080;
    let mut external = false;

    // let path = PathBuf::from(DIR);
    // println!(
    //     "Public directory: {}",
    //     path.canonicalize().unwrap().to_str().unwrap()
    // );

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
        } else {
            eprintln!("argument not recognized: {arg:?}")
        }
    }

    let port_range_end = port.checked_add(9).unwrap_or(u16::MAX);
    let server_thread = thread::spawn(move || {
        let listener = match TcpListener::bind(
            &(port..=port_range_end)
                .map(|port| {
                    if external {
                        std::net::SocketAddr::from((std::net::Ipv6Addr::UNSPECIFIED, port))
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
                    // thread::spawn(|| -> Result<(), ()> {
                    //     if let Ok(mut req) = stream.try_into() {
                    //         if let Ok(_) = new_handle(&req).respond_to(&mut req) {
                    //             return Ok(());
                    //         }
                    //     }
                    //     return Err(());
                    //     // let mut req: Request = stream.try_into().unwrap();
                    // });
                    thread::spawn(move || {
                        match handle(stream) {
                            Ok(_) => {}
                            // only panics current thread
                            Err(e) => panic!("{e:?}"),
                        }
                    });
                    eprintln!("Handed request off to thread.")
                }
                Err(e) => {
                    println!("Failed to read stream, {e:?}");
                }
            }
        }
    });

    let mut stdin = std::io::stdin().lock();

    loop {
        let mut buf = [0];
        stdin.read_exact(&mut buf).unwrap();
        if buf[0] == b'q' {
            break;
        } else {
            println!(">> [info] q to quit");
        }
    }

    drop(stdin);
    drop(server_thread);

    Ok(())
}

type IOErr = std::io::Error;
convert_err_to! {
    HandleErr <- RequestFromTcpStreamError, IOErr
}

// const DIR: &str = "./public/";
// fn new_handle(req: &Request) -> Response {
//     // FIXME: the `/` at the start of path overrides relativeness, trying to resolve to /file instead of ./public/file
//     let path = Path::new(DIR).join(req.pathname.clone());

//     println!("{}", path.to_str().unwrap());
//     panic!();

//     #[allow(unreachable_code)]
//     if !path.exists() {
//         Response::builder()
//             .status(404)
//             .body(Body::Data(format!(
//                 r#"File at "{}" not found."#,
//                 path.to_str().unwrap()
//             )))
//             .build()
//     } else if path.is_dir() {
//         // TODO: impl indexing / display index file

//         Response::builder()
//             .status(404)
//             .body(Body::Data(format!(
//                 r#"File at "{}" not found."#,
//                 path.to_str().unwrap()
//             )))
//             .build()
//     } else if path.is_file() {
//         let mut out = String::new();
//         std::fs::File::open(path)
//             .unwrap()
//             .read_to_string(&mut out)
//             .unwrap();

//         Response::builder()
//             .status(200)
//             .body(Body::Data(out))
//             .build()
//     } else {
//         unreachable!()
//     }
// }

fn handle(stream: TcpStream) -> Result<(), HandleErr> {
    let mut req = Request::try_from(stream)?;

    match &*req.pathname.to_lowercase() {
        "/form" => {
            let body = r#"<html dir="ltr" lang="en">
<head>
  <meta charset="utf-8" />
  <style></style>
</head>
<body>
  <form method="POST" action="/">
    <textarea name="body" placeholder="body goes here..."></textarea>
    <button type="submit" onclick="(e)=>{
        e.preventDefault();
        fetch('/').then((b) => {b.text()}).then(console.info);
    }">Send it</button>
  </form>
</body>
</html>
"#;
            let headers = format!(
                r"HTTP/1.1 200 Ok
Content-Type: text/html; charset=utf-8
Content-Length: {content_length}
",
                content_length = body.len()
            );

            write!(req, "{headers}\n{body}")?;
        }
        "/executable" => {
            match std::fs::read("./target/release/rust-http-server") {
                Ok(file) => {
                    write!(
                        req,
                        r"HTTP/1.1 200 Ok
Content-Type: application/octet-stream
Content-Length: {}

",
                        file.len()
                    )
                    .expect("Failed to write headers");
                    req.write_all(&file)
                    // .expect("Failed to write file.")
                }
                Err(e) => {
                    println!("error ocurred while opening file: {e:?}");
                    write!(req, "HTTP/1.1 500 Server Error\n\n")
                }
            }?
        }
        "/text" => match req.method {
            Method::Get => {
                let body = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <style>
    :root {{
        color-scheme: only dark;
    }}
  </style>
</head>
<body>
  <form method="POST" action="/text">
    <textarea name="text" placeholder="text here..."></textarea>
    <input type="submit" />
  </form>
</body>
</html>
"#;

                write!(
                    req,
                    r"HTTP/1.1 200 Ok
Content-Type: text/html; charset=utf-8
Content-Length: {}

{}",
                    body.len(),
                    body
                )?;
            }
            Method::Post => match (req.headers.get("content-type"), &req.body) {
                (Some(ct), Body::Data(data)) if ct == "application/x-www-form-urlencoded" => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .to_string();

                    let mut file = std::fs::File::create(format!("./out/{timestamp}.txt"))?;

                    match file.write(data.split_once("text=").unwrap().1.as_bytes()) {
                        Ok(_) => {
                            write!(req, "HTTP/1.1 200 No Content\n\nthank you. -- {timestamp}")?
                        }
                        Err(e) => {
                            println!("Failed to write to disk: {e:?}");
                            write!(req, "HTTP/1.1 500 Server Error\n\nsomething went wrong.")?
                        }
                    }
                }
                (ct, bd) => {
                    let headers = format!("{}", req.headers);
                    write!(
                        req,
                        "HTTP/1.1 400 User Error\n\nHeader is {}\nBody is {}\n\n{headers}",
                        if ct.is_none() { "Empty" } else { "Ok" },
                        match bd {
                            Body::Data(_) => "[[filled]]",
                            _ => "[[Empty]]",
                        }
                    )?;
                }
            },
            _ => {
                writeln!(req, "HTTP/1.1 505 Method Not Allowed")?;
            }
        },
        "/count-to-max" => {
            let mut num = 0;
            let then = SystemTime::now();

            loop {
                num += 1;
                if num == u32::MAX {
                    break;
                }
            }

            let now = SystemTime::now();
            let dur = now.duration_since(then).unwrap();

            let millis = dur.as_millis();
            write!(
                req,
                r"HTTP/1.1 200 Ok
    Content-Type: text/html; charset=utf-8

    <h1>Good job!</h1>
    <p>That took {millis} milliseconds to load.</p>
    <style>
    :root {{
    font-family: ui-sans, system-ui, sans-serif;
    color-scheme: only dark;
    }}
    </style>
    "
            )?;
        }
        "/" => {
            write!(
                req,
                r"HTTP/1.1 200 Ok
Content-Type: text/html; charset=utf-8

<h1>Nothing to see here, officer.</h1>
<style>
  :root {{
    font-family: ui-sans, system-ui, sans-serif;
    color-scheme: only dark;
  }}
</style>
"
            )?;
        }
        path => {
            println!("Headers:\n{}\n-- end headers --", req.headers.to_string());
            write!(
                req,
                "HTTP/1.1 404 Not Found\nContent-Type: text/plain; charset=utf-8\n\nFile at location \"{path}\" not found\n"
            )?;
        }
    }

    Ok(())
}
