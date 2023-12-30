//! Default handlers for [`Server`](crate::server::Server)

use crate::{
    request::{Body, Request},
    response::Response,
};
use std::path::Path;

pub enum IndexStyle {
    NotFound,
    IndexDirectory,
    IndexFile(String),
}

/// structure of a handler
pub type Handler = dyn Fn(&Request) -> Response;

pub(crate) fn not_found_handler_default(_: &Request) -> Response {
    let body = Body::Data(include_str!("./default_pages/404.html").to_string());

    Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .header("Content-Length", body.len())
        .body(body)
        .build()
}

pub fn fs_handler(directory: &str) -> Handler {
    const DIR: &str = "./public/";
    |req| {
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
            match *INDEX_STYLE.read().expect("failed to get index style") {
                IndexStyle::IndexDirectory => match std::fs::read_dir(path) {
                    Ok(files) => {
                        let files = files
                            .filter_map(|file| Some(file.ok()?.file_name().to_str()?.to_string()))
                            .fold(String::new(), |mut out, name| {
                                out +=
                                    format!(r#"<li><a href="./{name}">{name}</a></li>"#).as_str();
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
}