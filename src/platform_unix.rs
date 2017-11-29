use std::env;
use std::io::{Read, Result, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub struct ProxySocket(UnixStream);

impl ProxySocket {
	pub fn connect() -> Result<ProxySocket> {
		let user = env::var("USER").unwrap();
		let s = UnixStream::connect(format!("/tmp/keepassxc-{}.socket", user))?;
		let timeout: Option<Duration> = Some(Duration::from_secs(1));
		s.set_read_timeout(timeout)?;
		Ok(ProxySocket(s))
	}
}

impl Read for ProxySocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		self.0.read(buf)
	}
}

impl Write for ProxySocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> Result<()> {
		self.0.flush()
	}
}
