//! DIY server

use crate::{
    common::{Handler, Method},
    request::Request,
    response::Response, handlers::not_found_handler_default,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
    thread,
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
    routes: HashMap<Route, Arc<&'a Handler>>,
    not_found_handler: Option<Arc<&'a Handler>>,
}

// wild.
unsafe impl<'a> Sync for Server<'a> {}
unsafe impl<'a> Send for Server<'a> {}

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
                    Arc::new(handler),
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
                    Arc::new(handler),
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

    pub fn serve(self, address: &str, port: u16) -> ! {
        use std::net::TcpListener;

        let listener = TcpListener::bind(format!("{address}:{port}")).unwrap();

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

        loop {}
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
            None => {
                match self.not_found_handler.as_ref() {
                    Some(handler) => handler(req),
                    None => not_found_handler_default(req),
                }
            }
        }
    }
}
