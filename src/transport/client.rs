use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::common::error::{Error, Result};
use crate::transport::bridge::{Request, Response};
use crate::transport::ipc::Endpoint;

pub struct Client {
    endpoint: Endpoint,
}

impl Client {
    pub fn new(socket: impl Into<PathBuf>) -> Self {
        Client {
            endpoint: Endpoint::new(socket),
        }
    }

    pub fn socket(&self) -> &Path {
        self.endpoint.path()
    }

    /// The address that exists on this platform: a socket path, or a pipe name.
    pub fn address(&self) -> String {
        self.endpoint.address()
    }

    pub fn is_running(&self) -> bool {
        self.ping(Duration::from_millis(800))
    }

    /// Whether something is listening on the socket, even if it never answers.
    /// Only for diagnostics: a connectable socket is not proof of a healthy UI.
    pub fn is_connectable(&self) -> bool {
        self.endpoint.is_connectable()
    }

    pub fn ping(&self, timeout: Duration) -> bool {
        match self.call_with_timeout("ping", Value::Null, timeout) {
            Ok(value) => value.get("pong").and_then(Value::as_bool) == Some(true),
            Err(_) => false,
        }
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value> {
        self.call_with_timeout(method, params, Duration::from_secs(20))
    }

    pub fn call_with_timeout(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value> {
        let request = Request {
            id: uuid::Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        };
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');

        let response_line = self.endpoint.exchange(&line, timeout).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::ConnectionRefused
            {
                Error::UiNotRunning
            } else if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) {
                Error::Timeout
            } else {
                Error::Io(error)
            }
        })?;
        if response_line.trim().is_empty() {
            return Err(Error::Other("ui closed the connection".to_string()));
        }
        let response: Response = serde_json::from_str(response_line.trim())?;
        response.into_result()
    }

    pub fn wait_ready(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.ping(Duration::from_millis(500)) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(120));
        }
        self.ping(Duration::from_millis(500))
    }
}
