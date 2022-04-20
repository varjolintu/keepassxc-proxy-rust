use std::env;
use std::io::{self, Read, Write};
use std::path::Path;

#[cfg(not(windows))]
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use nix::sys::socket;
use nix::sys::socket::sockopt::SndBuf;
use nix::sys::socket::sockopt::RcvBuf;

#[cfg(windows)]
use named_pipe::PipeClient;

pub struct ProxySocket<T> {
    inner: T,
}

impl<R: Read> Read for ProxySocket<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<W: Write> Write for ProxySocket<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(windows)]
pub fn connect(buffer_size: u32) -> io::Result<ProxySocket<PipeClient>> {
    let username = env::var("USERNAME").unwrap();
    let pipe_name = format!("\\\\.\\pipe\\keepassxc\\{}\\org.keepassxc.KeePassXC.BrowserServer", username);
    let client = PipeClient::connect(pipe_name)?;
    Ok(ProxySocket { inner: client })
}

#[cfg(not(windows))]
pub fn connect(buffer_size: u32) -> io::Result<ProxySocket<UnixStream>> {
    use std::time::Duration;

    let socket_name = "org.keepassxc.KeePassXC.BrowserServer";
    let socket: String;
    if let Ok(xdg_run_dir) = env::var("XDG_RUNTIME_DIR") {
        let flatpak_run_dir = format!("{}/app/org.keepassxc.KeePassXC", xdg_run_dir);
        if Path::new(&flatpak_run_dir).is_dir() {
            socket = format!("{}/{}", flatpak_run_dir, socket_name);
        } else {
            socket = format!("{}/{}", xdg_run_dir, socket_name);
        }
    } else if let Ok(tmpdir) = env::var("TMPDIR") {
        socket = format!("{}/{}", tmpdir, socket_name);
    } else {
        socket = format!("/tmp/{}", socket_name);
    }
    let s = UnixStream::connect(socket)?;
    socket::setsockopt(s.as_raw_fd(), SndBuf, &(buffer_size as usize)).expect("setsockopt for SndBuf failed");
    socket::setsockopt(s.as_raw_fd(), RcvBuf, &(buffer_size as usize)).expect("setsockopt for RcvBuf failed");
    let timeout: Option<Duration> = Some(Duration::from_secs(1));
    s.set_read_timeout(timeout)?;
    Ok(ProxySocket { inner: s })
}
