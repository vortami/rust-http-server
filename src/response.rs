use std::{
    fmt::Display,
    io::{Result, Write},
};

use crate::{
    common::{Headers, HeadersBuilder},
    request::{Body, Request},
};

pub struct Status {
    pub code: u16,
    pub message: Option<String>,
}

impl From<u16> for Status {
    fn from(code: u16) -> Self {
        Self {
            code: code.into(),
            message: match code {
                200 => Some("Ok"),
                204 => Some("No Content"),
                404 => Some("Not Found"),
                _ => None,
            }
            .map(|s| s.to_string()),
        }
    }
}

impl<S: ToString> From<(u16, S)> for Status {
    fn from((code, message): (u16, S)) -> Self {
        Self {
            code,
            message: Some(message.to_string()),
        }
    }
}

// impl<N: Into<u16>, S: ToString> From<(N, S)> for Status {
//     fn from((code, message): (N, S)) -> Self {
//         Self {
//             code: code.into(),
//             message: Some(message.to_string()),
//         }
//     }
// }

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.code, {
            let str = self.message.clone().unwrap_or_default();
            format!("{}{}", if str.is_empty() { "" } else { " " }, str)
        })
    }
}
pub struct Response {
    pub status: Status,
    pub headers: Headers,
    pub body: Body,
}

impl Response {
    pub fn respond_to(self, req: &mut Request) -> Result<()> {
        write!(
            req,
            "HTTP/1.1 {status}\n{headers}\n\n{body}",
            status = self.status,
            headers = self.headers,
            // TODO:
            body = match self.body {
                Body::Data(d) => d,
                Body::Empty => String::new(),
            }
        )
        .map(|_| ())
    }

    pub fn status(&mut self, status: impl Into<Status>) {
        self.status = status.into()
    }

    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: None,
            headers: Headers::builder(),
            body: Body::Empty,
        }
    }
}

pub struct ResponseBuilder {
    status: Option<Status>,
    headers: HeadersBuilder,
    body: Body,
}

impl ResponseBuilder {
    pub fn status(mut self, status: impl Into<Status>) -> Self {
        self.status = Some(status.into());
        self
    }

    // pub fn status(mut self, code: u16, message: impl ToString) -> Self {
    //     self.status = Some(Status {
    //         code,
    //         message: Some(message.to_string()),
    //     });
    //     self
    // }

    // pub fn status_code(mut self, code: u16) -> Self {
    //     self.status = match self.status {
    //         Some(Status { message, .. }) => Some(Status { code, message }),
    //         None => Some(Status {
    //             code,
    //             message: None,
    //         }),
    //     };
    //     self
    // }

    // pub fn header(mut self, key: impl Into<HeaderKey>, value: impl ToString) -> Self {
    //     // (&mut self.headers).set(key, value);
    //     self
    // }

    pub fn headers(mut self, headers_fn: fn(HeadersBuilder) -> HeadersBuilder) -> Self {
        self.headers = headers_fn(self.headers);
        self
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self
    }

    pub fn build(self) -> Response {
        Response {
            body: self.body,
            headers: self.headers.build(),
            status: self.status.unwrap_or_else(|| Status {
                code: 200,
                message: Some("Ok".to_string()),
            }),
        }
    }
}
