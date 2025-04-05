use std::env;
use std::io::{self, Read, Write};

#[cfg(not(windows))]
use {
    nix::sys::socket::{
        setsockopt,
        sockopt::{RcvBuf, SndBuf},
    },
    std::os::unix::net::UnixStream,
    std::path::PathBuf,
};

#[cfg(windows)]
use named_pipe::PipeClient;

pub struct ProxySocket<T> {
    inner: T,
}

#[cfg(not(windows))]
impl ProxySocket<UnixStream> {
    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        let inner = self.inner.try_clone()?;
        Ok(Self { inner })
    }
}
#[cfg(windows)]
impl ProxySocket<PipeClient> {
    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        todo!();
    }
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
pub fn connect(_buffer_size: usize) -> io::Result<ProxySocket<PipeClient>> {
    let username = env::var("USERNAME").unwrap();
    let pipe_name = format!(
        "\\\\.\\pipe\\keepassxc\\{}\\org.keepassxc.KeePassXC.BrowserServer",
        username
    );
    let client = PipeClient::connect(pipe_name)?;
    Ok(ProxySocket { inner: client })
}

#[cfg(not(windows))]
/// Returns the directories where the socket could possible be located.
///
/// These directories should be tried in sequence, until one of them is found
/// to contain the socket.
fn get_socket_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if !cfg!(target_os = "macos") {
        if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
            let xdg_runtime_dir: PathBuf = dir.into();

            // Sandbox-friendly path.
            // Used in KeePassXC >= 2.7.2 and for all versions on Flatpak.
            dirs.push(xdg_runtime_dir.join("app/org.keepassxc.KeePassXC/"));

            // Legacy path.
            // Used by KeePassXC < 2.7.2.
            dirs.push(xdg_runtime_dir);
        };
    };

    // Default for macOS, and final fallback for Linux.
    dirs.push(env::temp_dir());

    dirs
}

#[cfg(not(windows))]
pub fn connect(buffer_size: usize) -> io::Result<ProxySocket<UnixStream>> {
    let socket_name = "org.keepassxc.KeePassXC.BrowserServer";
    let dirs = get_socket_dirs();
    let s = dirs
        .iter()
        .find_map(|dir| UnixStream::connect(dir.join(socket_name)).ok())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    setsockopt(&s, SndBuf, &buffer_size)?;
    setsockopt(&s, RcvBuf, &buffer_size)?;

    // Make sure reads are blocking.
    s.set_nonblocking(false)?;

    Ok(ProxySocket { inner: s })
}
