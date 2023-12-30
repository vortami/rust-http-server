//! All functions relating to responses

use crate::{
    common::{HeaderKey, Headers, HeadersBuilder},
    request::{Body, Request},
};
use std::{
    fmt::Display,
    io::{Result, Write},
};

/// Status Code/Message pair
pub struct Status {
    /// Code of the status
    pub code: u16,
    /// Message of the status
    pub message: Option<String>,
}

impl From<u16> for Status {
    fn from(code: u16) -> Self {
        Self {
            code,
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

/// Response structure
pub struct Response {
    /// Status of the response
    pub status: Status,
    /// Headers of the response
    pub headers: Headers,
    /// Body of the response
    pub body: Body,
}

impl Response {
    /// Same as `.respond_to()`, except it borrows [`Request`]
    pub fn respond_to_mut(self, req: &mut Request) -> Result<()> {
        write!(
            req,
            "HTTP/1.1 {status}\n{headers}\n{body}",
            status = self.status,
            headers = self.headers,
            body = match self.body {
                Body::Data(d) => d,
                Body::Empty => String::new(),
            }
        )
        .map(|_| ())
    }

    /// Respond to a [`Request`]
    pub fn respond_to(self, mut req: Request) -> Result<()> {
        self.respond_to_mut(&mut req)
    }

    /// Get the [`ResponseBuilder`]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: None,
            headers: Headers::builder(),
            body: Body::Empty,
        }
    }
}

/// Builder for [`Response`]
pub struct ResponseBuilder {
    status: Option<Status>,
    headers: HeadersBuilder,
    body: Body,
}

impl ResponseBuilder {
    /// Set the status of the response
    pub fn status(mut self, status: impl Into<Status>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Set a single header
    pub fn header(mut self, key: impl Into<HeaderKey>, value: impl ToString) -> Self {
        self.headers.insert(key.into(), value.to_string());
        self
    }

    /// Set multiple headers
    /// 
    /// # Examples
    /// ```
    /// # use rust_http_server::response::Response;
    /// Response::builder()
    ///     .headers(|headers| headers
    ///         .set("foo", "bar")
    ///         .set("baz", "123")
    ///     );
    /// ```
    /// ```
    /// # use rust_http_server::response::Response;
    /// let builder = Response::builder();
    /// let vec = vec![("Content-Type", "application/json"), ("Content-Length", "1024")];
    /// 
    /// Response::builder()
    ///     .headers(|header| vec.iter().fold(header, |headers, (k, v)| {
    ///         headers.set(k, v)
    ///     }));
    /// ```
    pub fn headers(mut self, headers_fn: impl Fn(HeadersBuilder) -> HeadersBuilder) -> Self {
        self.headers = headers_fn(self.headers);
        self
    }


    /// Set the body of the response.
    ///
    /// `body` can be `()` or anything that implements [`ToString`]
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self
    }


    /// Construct a [`Response`]
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
