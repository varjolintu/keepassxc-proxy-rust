extern crate byteorder;

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::net::UnixListener;
use std::str;
use std::thread;
use std::time::Duration;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};

fn valid_length(length: u32) -> bool
{
	return length > 0 && length <= 4096;	// 1024 ^ 2 is the maximum
}

fn read_header() -> (u32)
{
	let stdin = io::stdin();
	let mut buf = vec![0; 4];
	let mut handle = stdin.lock();

	handle.read_exact(&mut buf).unwrap();
	let length: u32 = NativeEndian::read_u32(&buf);
	return length;
}

fn read_body(length: u32, mut socket: &UnixStream)
{
	let mut buffer = vec![0; length as usize];
	let stdin = io::stdin();
	let mut handle = stdin.lock();

	match handle.read_exact(&mut buffer) {
		Ok(_v) => {
			if valid_length(length) {
				socket.write(&buffer).unwrap();
				socket.flush().unwrap();
				read_unix_response(length, &socket);
			}
		},
		Err(_e) => {}
	}
}

fn read_unix_response(length: u32, mut socket: &UnixStream)
{
	let mut buf = vec![0; length as usize];

    match socket.read(&mut buf) {
		Ok(_length) => {
			let text = str::from_utf8(&buf).unwrap();
			write_output(text);
		},
		Err(_e) => {}
    }
}

fn write_output(text: &str)
{
	let textlen = text.len();
	let stdout = io::stdout();
	let mut handle = stdout.lock();

	handle.write_u32::<NativeEndian>(textlen as u32).unwrap();
	handle.write(text.as_bytes()).unwrap();
}

fn main() {
	let socket = UnixStream::connect("/tmp/kpxc_server").unwrap();
	let timeout: Option<Duration> = Some(Duration::from_secs(1));
	socket.set_read_timeout(timeout).unwrap();

	// Start thread for user input reading
	let send_socket = socket.try_clone().expect("Cannot clone socket");
	let ui = thread::spawn(move || {
		loop {
			let length = read_header();
			read_body(length, &send_socket);
	    }
    });

    let _ui_res = ui.join();
}
