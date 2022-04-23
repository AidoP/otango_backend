use std::fmt;
use serde::Deserialize;

pub enum Error {
    Net(gloo_net::Error),
    Json(serde_json::Error),
    NotFound
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Net(e) => write!(f, "{}", e),
            Self::Json(e) => write!(f, "{}", e),
            Self::NotFound => f.write_str("Not Found")
        }
    }
}
impl From<gloo_net::Error> for Error {
    fn from(error: gloo_net::Error) -> Self {
        Self::Net(error)
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// Lazily-loaded data from an external source
pub enum LazyData<T> {
    Loading,
    Ready(T),
    Error(Error)
}
impl<T> LazyData<T> {
    pub fn map<U, L: FnOnce() -> U, R: FnOnce(&T) -> U, E: FnOnce(&Error) -> U>(&self, loading: L, ready: R, error: E) -> U {
        match self {
            Self::Loading => loading(),
            Self::Ready(data) => ready(data),
            Self::Error(e) => error(&e)
        }
    }
    pub fn map_ok<U, L: FnOnce(bool) -> U, R: FnOnce(&T) -> U, E: FnOnce(&Error) -> U>(&self, not_ready: L, ready: R) -> U {
        match self {
            Self::Loading => not_ready(false),
            Self::Error(_) => not_ready(true),
            Self::Ready(data) => ready(data)
        }
    }
    pub fn map_ok_either<U, V, L: FnOnce(V) -> U, R: FnOnce(&T) -> U>(&self, not_ready: L, ready: R, loading: V, error: V) -> U {
        match self {
            Self::Loading => not_ready(loading),
            Self::Error(_) => not_ready(error),
            Self::Ready(data) => ready(data)
        }
    }
}
impl<T: for<'de> Deserialize<'de>> LazyData<T> {
    pub fn load<F: 'static + FnOnce(Self)>(url: &str, and_then: F) -> Self {
        let request = gloo_net::http::Request::new(url);
        wasm_bindgen_futures::spawn_local(async move {
            let response = request.send().await
                .map_err(|e| e.into())
                .and_then(|r| if r.status() == 404 { Err(Error::NotFound) } else { Ok(r) });
            let data = match response {
                Ok(response) => match response.json().await {
                    Ok(data) => Self::Ready(data),
                    Err(error) => Self::Error(error.into())
                },
                Err(error) => Self::Error(error.into())
            };
            and_then(data);
            //data.set(component::Word::get(properties.word));
        });
        Self::Loading
    }
}