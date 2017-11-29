extern crate byteorder;
#[cfg(windows)]
extern crate named_pipe;

use std::io::{self, Read, Write};
use std::thread;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};

#[cfg(not(windows))]
#[path = "platform_unix.rs"]
mod platform;

#[cfg(windows)]
#[path = "platform_windows.rs"]
mod platform;

use platform::ProxySocket;

fn valid_length(length: u32) -> bool
{
	return length > 0 && length <= 4096;	// 1024 ^ 2 is the maximum
}

fn read_header() -> u32
{
	let stdin = io::stdin();
	let mut buf = vec![0; 4];
	let mut handle = stdin.lock();

	handle.read_exact(&mut buf).unwrap();
	NativeEndian::read_u32(&buf)
}

fn read_body(length: u32, socket: &mut ProxySocket)
{
	let mut buffer = vec![0; length as usize];
	let stdin = io::stdin();
	let mut handle = stdin.lock();

	if let Ok(_) = handle.read_exact(&mut buffer) {
		if valid_length(length) {
			socket.write_u32::<NativeEndian>(length).unwrap();
			socket.write(&buffer).unwrap();
			socket.flush().unwrap();
			read_response(socket);
		}
	}
}

fn read_response(socket: &mut ProxySocket)
{
	let mut len_buf = vec![0; 4];

	if let Ok(_) = socket.read_exact(&mut len_buf) {
		let len = NativeEndian::read_u32(&len_buf);
		let mut buf = vec![0; len as usize];
		if let Ok(_) = socket.read_exact(&mut buf) {
			write_response(&buf, len);
		}
	}
}

fn write_response(buf: &[u8], len: u32) {
	let stdout = io::stdout();
	let mut out = stdout.lock();

	out.write_u32::<NativeEndian>(len).unwrap();
	out.write(buf).unwrap();
	out.flush().unwrap();
}

fn main() {
	let mut socket = ProxySocket::connect().unwrap();

	// Start thread for user input reading
	//let mut send_socket = socket.try_clone().expect("Cannot clone socket");
	let ui = thread::spawn(move || {
		loop {
			let length = read_header();
			read_body(length, &mut socket);
		}
	});

	let _ui_res = ui.join();
}
