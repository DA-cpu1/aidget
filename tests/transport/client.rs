#![cfg(unix)]

//! The client's timeouts and error mapping against a fake bridge peer.
//!
//! A named pipe is the Windows transport, so this file builds its mock on a Unix
//! socket and only runs there; `#[cfg(unix)]` keeps the crate checkable on
//! every target.

use aidget::transport::client::Client;
use std::time::Duration;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};

fn socket_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn missing_socket_is_not_running() {
    let dir = socket_dir();
    let client = Client::new(dir.path().join("ui.sock"));
    assert!(!client.is_running());
}

#[test]
fn connected_but_silent_socket_is_not_running() {
    let dir = socket_dir();
    let path = dir.path().join("ui.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let _server = std::thread::spawn(move || {
        let (_stream, _) = listener.accept().unwrap();
        std::thread::sleep(Duration::from_secs(2));
    });

    let client = Client::new(&path);
    assert!(!client.is_running());
}

#[test]
fn ping_requires_a_pong_response() {
    let dir = socket_dir();
    let path = dir.path().join("ui.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let _server = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            respond(stream, r#"{"id":"x","ok":true,"result":{"pong":true}}"#);
        }
    });

    assert!(Client::new(&path).ping(Duration::from_millis(500)));
}

fn respond(mut stream: UnixStream, body: &str) {
    let mut line = String::new();
    BufReader::new(stream.try_clone().unwrap())
        .read_line(&mut line)
        .unwrap();
    stream.write_all(body.as_bytes()).unwrap();
    stream.write_all(b"\n").unwrap();
}
