//! All functions relating to requests

use crate::common::{Headers, Method, Search};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

/// Request structure
pub struct Request {
    /// Method of the request
    pub method: Method,
    /// Pathname of the request
    pub pathname: String,
    /// Search string of the request
    pub search: Search,
    /// Headers of the request
    pub headers: Headers,
    /// Body of the request
    pub body: Body,
    pub(crate) stream: TcpStream,
}

impl Write for Request {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

/// Body for [`Request`]/[`Response`]
pub enum Body {
    /// Body has data
    Data(String),
    // #[cfg(feature = "streamed_response")] Streamed(TcpStream),
    /// Body has no data
    Empty,
}

impl Body {
    /// Get the length of the body (`0` if [`Body::Empty`])
    pub fn len(&self) -> usize {
        match self {
            Self::Data(data) => data.len(),
            Self::Empty => 0,
        }
    }

    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Data(data) => data.len() == 0,
            Self::Empty => true,
        }
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        Self::Data(value.to_string())
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Self::Data(value)
    }
}

impl TryFrom<TcpStream> for Request {
    type Error = std::convert::Infallible;

    fn try_from(value: TcpStream) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value);

        let mut read_line = || {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Ok(..) => {
                    if buf.ends_with("\r\n") {
                        buf[..buf.len() - 2].to_string()
                    } else if buf.ends_with('\n') {
                        buf[..buf.len() - 1].to_string()
                    } else {
                        buf
                    }
                }
                Err(e) => panic!("{e}"),
            }
        };

        let (method, (pathname, search)) = {
            let copy = read_line();
            let mut line = copy.split(' ');

            (line.next().unwrap().parse::<Method>().unwrap(), {
                let path = line.next().unwrap_or_else(|| {
                    println!(
                        r#"ERROR: {{
  "path is None"
  {copy}
}}"#
                    );

                    "/"
                });
                match path.split_once('?') {
                    Some((l, r)) => (
                        l.to_string(),
                        r.parse().expect("Could not parse search string"),
                    ),
                    None => (path.to_string(), Search::default()),
                }
            })
        };

        let mut headers = HashMap::new();
        loop {
            let line = read_line();
            if line.trim().is_empty() {
                break;
            }
            let (h, n) = line.split_once(": ").unwrap();
            headers.insert(h.to_owned(), n.to_owned());
        }
        let headers = Headers::from(headers);

        let body = match method {
            Method::Post => match headers.get("Content-Length") {
                Some(content_length) => {
                    let content_length = content_length
                        .parse::<usize>()
                        .expect("Content-Length header does not contain a number");

                    let mut buf = vec![0; content_length];
                    match reader.read_exact(&mut buf) {
                        Ok(_) => Body::Data(
                            String::from_utf8(buf).expect("Failed to parse body to string."),
                        ),
                        Err(e) => panic!("{e:?}"),
                    }
                }
                None => Body::Empty,
            },
            _ => Body::Empty,
        };

        Ok(Self {
            method,
            pathname,
            search,
            headers,
            body,
            stream: reader.into_inner(),
        })
    }
}
