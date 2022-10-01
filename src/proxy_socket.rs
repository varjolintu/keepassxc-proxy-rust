use std::env;
use std::io::{self, Read, Write, Error, ErrorKind};

#[cfg(not(windows))]
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use nix::sys::socket;
use nix::sys::socket::sockopt::SndBuf;
use nix::sys::socket::sockopt::RcvBuf;

#[cfg(windows)]
use named_pipe::PipeClient;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Op};

const SOCKET_NAME : &str = "org.keepassxc.KeePassXC.BrowserServer";

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
pub fn connect(buffer_size: usize) -> io::Result<ProxySocket<PipeClient>> {
    let username = env::var("USERNAME").unwrap();
    let pipe_name = format!("\\\\.\\pipe\\keepassxc\\{}\\org.keepassxc.KeePassXC.BrowserServer", username);
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

// Connects to the socket or waits for it.
//
// If the socket already exists, connect to it immediately, otherwise, wait
// until it appears, and then connect to it.
//
// This function is blocking, and may block indefinitely if KeePassXC is never
// started.
fn get_or_await_socket() -> io::Result<UnixStream> {
    let dirs = get_socket_dirs();

    // Set up a watcher to await for the socket to be created.
    // Do this before checking for existing sockets to avoid race conditions.
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        Watcher::new_raw(tx).map_err(|err| Error::new(ErrorKind::Other, err))?;

    // If a socket exists, return that immediately:
    if let Some(socket) = dirs
        .iter()
        .find_map(|dir| {
            // Start watching this directory in case a socket is created from this moment onwards.
            if watcher.watch(dir, RecursiveMode::NonRecursive).is_err() {
                eprintln!("Failed to watch {:?}", dir);
            }

            // Check if the socket already exists.
            UnixStream::connect(dir.join(SOCKET_NAME)).ok()
        }) {
            return Ok(socket);
        };

    // Watch the candidate directories and await for the socket to be created.
    //
    // Note that if the socket was created after the beginning of this function and before the
    // above iteration, the watcher will have an even for this already.
    loop {
        match rx.recv() {
            Ok(notify::RawEvent{op: Ok(Op::CREATE), path: Some(path), ..}) => {
                if path.file_name().unwrap_or_default() == SOCKET_NAME {
                    return UnixStream::connect(path);
                }
            },
            Ok(_) => {},
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        }
    };
}

#[cfg(not(windows))]
pub fn connect(buffer_size: usize) -> io::Result<ProxySocket<UnixStream>> {
    let s = get_or_await_socket()?;

    socket::setsockopt(s.as_raw_fd(), SndBuf, &buffer_size)?;
    socket::setsockopt(s.as_raw_fd(), RcvBuf, &buffer_size)?;

    let timeout: Option<Duration> = Some(Duration::from_secs(1));
    s.set_read_timeout(timeout)?;
    Ok(ProxySocket { inner: s })
}
