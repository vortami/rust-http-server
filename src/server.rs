//! DIY server

use crate::{
    common::{Handler, Method},
    handlers::not_found_handler_default,
    request::Request,
    response::Response,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    thread,
    time::Duration,
};

struct Route {
    /// [`None`] matches ALL methods
    method: Option<Method>,
    path: String,
    case_sensitive: bool,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method && self.path == other.path
    }
}

impl Eq for Route {}

impl PartialEq<(Method, String)> for Route {
    fn eq(&self, (method, path): &(Method, String)) -> bool {
        (self.method.is_none() || self.method == Some(method.clone()))
            && if self.case_sensitive {
                &self.path == path
            } else {
                self.path.to_lowercase() == path.to_lowercase()
            }
    }
}

impl Hash for Route {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.path.to_lowercase().hash(hasher)
    }
}

#[derive(Default)]
/// Simple server implementation
pub struct Server<'a> {
    routes: HashMap<Route, &'a Handler>,
    not_found_handler: Option<&'a Handler>,
}

// wild.
unsafe impl<'a> Sync for Server<'a> {}

macro_rules! method_impl {
    ($($name: ident ($exact: ident) => $method: expr;)+) => {
        $(
            pub fn $name(mut self, path: impl ToString, handler: &'a Handler) -> Self {
                self.routes.insert(
                    Route {
                        method: Some($method),
                        path: path.to_string(),
                        case_sensitive: false,
                    },
                    handler,
                );
                self
            }

            pub fn $exact(mut self, path: impl ToString, handler: &'a Handler) -> Self {
                self.routes.insert(
                    Route {
                        method: Some($method),
                        path: path.to_string(),
                        case_sensitive: true,
                    },
                    handler,
                );
                self
            }
        )*
    }
}

#[allow(missing_docs, dead_code)]
impl<'a> Server<'a> {
    /// Create a new Server
    pub fn new() -> Self {
        Default::default()
    }

    method_impl! {
        get (get_exact) => Method::Get;
        post (post_exact) => Method::Post;
    }

    /// Can be used as a 404, but it's also repurposable as a catch-all!
    ///
    /// Using this while not matching on anything else will catch every incoming request.
    pub fn not_found(mut self, handler: &'a Handler) -> Self {
        self.not_found_handler = Some(handler);
        self
    }

    pub fn serve(self, address: &str, port: u16) -> ! {
        use std::net::TcpListener;

        let listener = TcpListener::bind(format!("{address}:{port}")).expect("Failed to bind");

        thread::scope(|scope| {
            for stream in listener.incoming() {
                scope.spawn(|| {
                    let req: Request = stream
                        .expect("failed to open stream")
                        .try_into()
                        .expect("failed to parse request");

                    self.handle(&req)
                        .respond_to(req)
                        .expect("failed to send response");
                });
            }
        });

        loop {
            thread::sleep(Duration::from_secs(u64::MAX))
        }
    }

    pub fn handle(&self, req: &Request) -> Response {
        let handler = self.routes.iter().find_map(|(route, handler)| {
            if route == &(req.method.clone(), req.pathname.clone()) {
                Some(handler)
            } else {
                None
            }
        });

        match handler {
            Some(handler) => handler(req),
            None => match self.not_found_handler.as_ref() {
                Some(handler) => handler(req),
                None => not_found_handler_default(req),
            },
        }
    }
}
