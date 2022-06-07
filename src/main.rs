extern crate byteorder;
#[cfg(windows)]
extern crate named_pipe;
extern crate nix;
extern crate notify;

use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use std::convert::TryInto;
use std::io::{stdin, stdout, Error, ErrorKind, Read, Result, Write};
use std::thread;

mod proxy_socket;

use proxy_socket::ProxySocket;

const BUFFER_SIZE: usize = 1024 ^ 2; // 1024 ^ 2 is the maximum

fn valid_length(length: usize) -> bool {
    length > 0 && length <= BUFFER_SIZE
}

// Read a header (message size) from stdin (e.g.: from the browser).
fn read_header() -> Result<usize> {
    let stdin = stdin();
    let mut buf = vec![0; 4];
    let mut handle = stdin.lock();

    handle.read_exact(&mut buf)?;

    NativeEndian::read_u32(&buf)
        .try_into()
        .map_err(|err| Error::new(ErrorKind::InvalidData, err))
}

// Handle a whole request/response cycle
//
// Read a message body from stdin (e.g.: from the browser), and echo it back to the browser's
// socket. Then await a response from the socket and relay that back to the browser.
fn read_body<T: Read + Write>(length: usize, socket: &mut ProxySocket<T>) -> Result<()> {
    let mut buffer = vec![0; length];
    let stdin = stdin();
    let mut handle = stdin.lock();

    handle.read_exact(&mut buffer)?;

    if valid_length(length) {
        socket.write_all(&buffer)?;
        socket.flush()?;
        read_response(socket)?;
    }

    Ok(())
}

// Read a response (from KP's socket) and echo it back to the browser.
fn read_response<T: Read>(socket: &mut ProxySocket<T>) -> Result<()>{
    let mut buf = vec![0; BUFFER_SIZE];
    if let Ok(len) = socket.read(&mut buf) {
        write_response(&buf[0..len])?;
    }

    Ok(())
}

// Write a response to stdout (e.g.: to the browser).
fn write_response(buf: &[u8]) -> Result<()> {
    let stdout = stdout();
    let mut out = stdout.lock();

    out.write_u32::<NativeEndian>(buf.len() as u32)?;
    out.write_all(buf)?;
    out.flush()?;

    Ok(())
}

fn main() {
    let mut socket = proxy_socket::connect(BUFFER_SIZE).unwrap();

    // Start thread for user input reading
    let ui = thread::spawn(move || loop {
        let length = read_header().unwrap();
        read_body(length, &mut socket).unwrap();
    });

    let _ui_res = ui.join().unwrap();
}
