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

// > The maximum size of a single message from the application is 1 MB.
//
// From: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging#app_side
const BUFFER_SIZE: usize = 1024 * 1024;

/// Reads from stdin and writes to the socket.
/// Returns on error.
fn stdin_to_socket<T: Read + Write>(socket: &mut ProxySocket<T>) -> Result<()> {
    let mut handle = stdin().lock();
    let mut len = vec![0; std::mem::size_of::<u32>()];

    loop {
        handle.read_exact(&mut len)?;
        let length: usize = NativeEndian::read_u32(&len)
            .try_into()
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;

        let mut buffer = vec![0; length];
        handle.read_exact(&mut buffer)?;

        socket.write_all(&buffer)?;
        socket.flush()?;
    }
}

/// Reads from the socket and writes to stdout.
/// Returns on error.
fn socket_to_stdout<T: Read + Write>(socket: &mut ProxySocket<T>) -> Result<()> {
    let mut out = stdout().lock();
    let mut buf = [0; BUFFER_SIZE];

    loop {
        if let Ok(len) = socket.read(&mut buf) {
            // If a message is larger than the maximum, ignore it entirely. These are disallowed
            // by the browser anyway, so sending one would be a protocol violation.
            if len <= BUFFER_SIZE {
                out.write_u32::<NativeEndian>(len as u32)?;
                out.write_all(&buf[..len])?;
                out.flush()?;
            };
        } else {
            // TOOD: is the socket is closed, we should try to reconnect.

            return Err(Error::from(ErrorKind::BrokenPipe));
        }
    }
}

fn main() -> Result<()> {
    let mut socket = proxy_socket::connect(BUFFER_SIZE)?;
    let mut socket_clone = socket.try_clone()?;

    thread::spawn(move || socket_to_stdout(&mut socket_clone).unwrap());

    // If stdin is closed, that means that Firefox has exited, so we exit too.
    // If the socket is closed, this will (eventually) fail too, however, this can later be
    // refactored to reconnect the underlying ProxySocket.
    stdin_to_socket(&mut socket).unwrap();

    Ok(())
}
