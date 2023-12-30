//! DIY server

use crate::{
    common::{Handler, Method},
    request::{Body, Request},
    response::Response,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
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
pub struct Server {
    routes: HashMap<Route, Handler>,
    not_found_handler: Option<Handler>,
}

fn not_found_handler_default(_: &Request) -> Response {
    let body = Body::Data(include_str!("./default_pages/404.html").to_string());

    Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .header("Content-Length", body.len())
        .body(body)
        .build()
}

#[allow(missing_docs, dead_code)]
impl Server {
    /// Create a new Server
    pub fn new() -> Self {
        Default::default()
    }

    pub fn route_sensitive(
        mut self,
        method: Method,
        route: impl ToString,
        handler: Handler,
    ) -> Self {
        self.routes.insert(
            Route {
                method: Some(method),
                path: route.to_string(),
                case_sensitive: true,
            },
            handler,
        );
        self
    }

    pub fn route(mut self, method: Method, route: impl ToString, handler: Handler) -> Self {
        self.routes.insert(
            Route {
                method: Some(method),
                path: route.to_string(),
                case_sensitive: false,
            },
            handler,
        );
        self
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

                    self.handle(&req).respond_to(req).expect("failed to send response");
                });
            }
        });

        loop {}
    }

    pub fn handle(&self, req: &Request) -> Response {
        let not_found: &Handler = &self.not_found_handler.unwrap_or(not_found_handler_default);

        let handler: &Handler = self.routes.iter().find_map(|(route, handler)| {
            if route == &(req.method.clone(), req.pathname.clone()) {
                Some(handler)
            } else {
                None
            }
        }).unwrap_or(not_found);

        handler(req)
    }
}
