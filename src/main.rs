extern crate byteorder;

use std::io::{self, Read, Write};
use std::net::UdpSocket;
use std::str;
use std::thread;
use std::time::Duration;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};

fn valid_length(length: u32) -> bool
{
	return length > 0 && length <= 16384;	// 1024 ^ 2 is the maximum
}

fn read_header() -> (u32)
{
	let stdin = io::stdin();
	let mut buf = vec![0; 4];
	let mut handle = stdin.lock();

	handle.read_exact(&mut buf);
	let length: u32 = NativeEndian::read_u32(&buf);
	return length;
}

fn read_body(length: u32, socket: &UdpSocket)
{
	let mut buffer = vec![0; length as usize];
	let stdin = io::stdin();
	let mut handle = stdin.lock();

	match handle.read_exact(&mut buffer) {
		Ok(v) => {
			if valid_length(length) {
				socket.send_to(&buffer, "127.0.0.1:19700").expect("Cannot send data");
			}
		},
		Err(e) => panic!("Read error: {}", e)
		//Err(e) => {}
	}
}

fn read_udp_response(socket: &UdpSocket)
{
	let mut buf = [0; 4069];

    match socket.recv_from(&mut buf) {
    	Ok((length, src)) => {
    		if valid_length(length as u32) {
    			thread::spawn(move || {
    				let buf = &mut buf[..length];
					let text = str::from_utf8(&buf).unwrap();
					write_output(text);
			    });
    		}
    	},
    	//Err(e) => panic!("Read error: {}", e)
    	Err(e) => {}
    }    
}

fn write_output(text: &str)
{
	let textlen = text.len();
	let stdout = io::stdout();
	let mut handle = stdout.lock();

	handle.write_u32::<NativeEndian>(textlen as u32).unwrap();
	handle.write(text.as_bytes());
}

fn main() { 

    let socket = UdpSocket::bind("127.0.0.1:0").expect("Couldn't bind to address");
    let timeout: Option<Duration> = Some(Duration::from_secs(1));
    socket.set_read_timeout(timeout);

    // Start thread for user input reading
    let send_socket = socket.try_clone().expect("Cannot clone socket");
    let ui = thread::spawn(move || {
    	loop {
	    	let length = read_header();
	    	read_body(length, &send_socket);
	    }
    });

    // Start thread for UDP packet receiving
    let recv_socket = socket.try_clone().expect("Cannot clone socket");
    let pr = thread::spawn(move || {
    	loop {
    		read_udp_response(&recv_socket);
    	}
    });

    let ui_res = ui.join();
    let pr_res = pr.join();
}