extern crate byteorder;
#[cfg(windows)]
extern crate named_pipe;
extern crate nix;

use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use std::convert::TryInto;
use std::io::{stdin, stdout, Error, ErrorKind, Read, Result, Write};
use std::thread;

mod proxy_socket;

use proxy_socket::ProxySocket;

const BUFFER_SIZE: usize = 1024 ^ 2;	 // 1024 ^ 2 is the maximum

fn valid_length(length: usize) -> bool {
    length > 0 && length <= BUFFER_SIZE
}

fn read_header() -> usize {
    let stdin = stdin();
    let mut buf = vec![0; 4];
    let mut handle = stdin.lock();

    handle.read_exact(&mut buf).unwrap();
    NativeEndian::read_u32(&buf).try_into().unwrap()
}

fn read_body<T: Read + Write>(length: usize, socket: &mut ProxySocket<T>) {
    let mut buffer = vec![0; length];
    let stdin = stdin();
    let mut handle = stdin.lock();

    if handle.read_exact(&mut buffer).is_ok() && valid_length(length) {
        socket.write_all(&buffer).unwrap();
        socket.flush().unwrap();
        read_response(socket);
    }
}

fn read_response<T: Read>(socket: &mut ProxySocket<T>) {
    let mut buf = vec![0; BUFFER_SIZE];
    if let Ok(len) = socket.read(&mut buf) {
        write_response(&buf[0..len]);
    }
}

fn write_response(buf: &[u8]) {
    let stdout = stdout();
    let mut out = stdout.lock();

    out.write_u32::<NativeEndian>(buf.len() as u32).unwrap();
    out.write_all(buf).unwrap();
    out.flush().unwrap();
}

fn main() {
    let mut socket = proxy_socket::connect(BUFFER_SIZE).unwrap();

    // Start thread for user input reading
    let ui = thread::spawn(move || {
        loop {
            let length = read_header();
            read_body(length, &mut socket);
        }
    });

    let _ui_res = ui.join().unwrap();
}
