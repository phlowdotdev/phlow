use valu3::prelude::*;

#[derive(Clone, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    TRACE,
    CONNECT,
    ANY,
}

#[derive(Clone, Debug)]
pub struct Route {
    path: String,
    method: Method,
}

impl From<Value> for Route {
    fn from(value: Value) -> Self {
        let path = value.get("path").unwrap().as_string();
        let method = value.get("method").unwrap().as_string();

        let method = match method.as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "OPTIONS" => Method::OPTIONS,
            "HEAD" => Method::HEAD,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            _ => Method::ANY,
        };

        Route { path, method }
    }
}

#[derive(Clone, Debug)]
pub struct Setup {
    route: Route,
    port: Option<u16>,
    address: Option<String>,
}

impl From<Value> for Setup {
    fn from(value: Value) -> Self {
        let route = {
            let route = value.get("route").unwrap().clone();
            Route::from(route)
        };

        let port = match value.get("port") {
            Some(port) => Some(port.to_u64().unwrap() as u16),
            None => None,
        };

        let address = match value.get("address") {
            Some(address) => Some(address.as_string()),
            None => None,
        };

        Setup {
            route,
            port,
            address,
        }
    }
}
