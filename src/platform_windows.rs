
use std::io::{Result, Read, Write};
use named_pipe::PipeClient;

pub struct ProxySocket(PipeClient);

impl ProxySocket {
	pub fn connect() -> Result<ProxySocket> {
		let client = PipeClient::connect("\\\\.\\pipe\\KeePassHttp").unwrap();
		Ok(ProxySocket(client))
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
