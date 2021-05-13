use std::{fs::remove_file, path::PathBuf, process};

pub fn get_socket_path() -> PathBuf {
    let socket_dir = std::env::var("STEWARDX_DIR").unwrap_or_else(|_| String::from("/tmp/"));
    let mut socket_path = PathBuf::from(socket_dir);
    socket_path.push("stewardx.sock");
    socket_path
}

pub fn cleanup() {
    let _ = remove_file(get_socket_path());
}

pub fn exit() {
    cleanup();
    process::exit(0);
}