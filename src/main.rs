extern crate byteorder;
#[cfg(windows)]
extern crate named_pipe;

use std::io::{self, Read, Write};
use std::thread;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};

mod proxy_socket;

use proxy_socket::ProxySocket;

fn valid_length(length: u32) -> bool {
	return length > 0 && length <= 4096; // 1024 ^ 2 is the maximum
}

fn read_header() -> u32 {
	let stdin = io::stdin();
	let mut buf = vec![0; 4];
	let mut handle = stdin.lock();

	handle.read_exact(&mut buf).unwrap();
	NativeEndian::read_u32(&buf)
}

fn read_body<T: Read + Write>(length: u32, socket: &mut ProxySocket<T>) {
	let mut buffer = vec![0; length as usize];
	let stdin = io::stdin();
	let mut handle = stdin.lock();

	if let Ok(_) = handle.read_exact(&mut buffer) {
		if valid_length(length) {
			socket.write(&buffer).unwrap();
			socket.flush().unwrap();
			read_response(socket);
		}
	}
}

fn read_response<T: Read>(socket: &mut ProxySocket<T>) {
	let mut buf = vec![0; 1024 * 1024];
	if let Ok(len) = socket.read(&mut buf) {
		// for some reason the length is 1 byte too long in linux?
		let mut adjust = 0;
		if cfg!(not(windows)) {
			adjust = 1;
		}
		write_response(&buf[0..len - adjust]);
	}
}

fn write_response(buf: &[u8]) {
	let stdout = io::stdout();
	let mut out = stdout.lock();

	out.write_u32::<NativeEndian>(buf.len() as u32).unwrap();
	out.write(buf).unwrap();
	out.flush().unwrap();
}

fn main() {
	let mut socket = proxy_socket::connect().unwrap();

	// Start thread for user input reading
	let ui = thread::spawn(move || {
		loop {
			let length = read_header();
			read_body(length, &mut socket);
		}
	});

	let _ui_res = ui.join().unwrap();
}
