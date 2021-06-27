#![doc(html_root_url = "https://docs.rs/tokio-anon-pipe/0.1.0")]
//! Asynchronous anonymous pipe for Windows.
//!
//! inspired by
//! <https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L48>
//!
//! > Note that we specifically do *not* use `CreatePipe` here because
//! > unfortunately the anonymous pipes returned do not support overlapped
//! > operations. Instead, we create a "hopefully unique" name and create a
//! > named pipe which has overlapped operations enabled.
//!
//! # Supported platform
//!
//! `x86_64-pc-windows-msvc` only
//!
//! # Example
//!
//! ```
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> anyhow::Result<()> {
//!     let (mut r, mut w) = tokio_anon_pipe::anon_pipe().await?;
//!
//!     w.write_all(b"HELLO, WORLD!").await?;
//!
//!     let mut buf = [0; 16];
//!     let len = r.read(&mut buf[..]).await?;
//!
//!     assert_eq!(&buf[..len], &b"HELLO, WORLD!"[..]);
//!     Ok(())
//! }
//! ```
use std::mem;
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle};
use std::pin::Pin;
use std::process;
use std::task::{Context, Poll};

#[cfg(not(windows))]
use stub::*;
use tokio::io;
#[cfg(windows)]
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

#[cfg(not(windows))]
mod stub {
    #![allow(unused_variables)]
    //! stub for non windows.
    //! developing reason.
    use super::*;

    pub(super) type HANDLE = *mut std::ffi::c_void;
    pub(super) type RawHandle = HANDLE;

    #[derive(Debug)]
    pub struct NamedPipeServer;

    pub(super) trait IntoRawHandle {
        fn into_raw_handle(self) -> RawHandle;
    }

    pub(super) trait AsRawHandle {
        fn as_raw_handle(&self) -> RawHandle;
    }

    impl NamedPipeServer {
        pub(super) async fn connect(&self) -> io::Result<()> {
            panic!("stub")
        }
    }

    impl io::AsyncRead for NamedPipeServer {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            panic!("stub")
        }
    }

    impl io::AsyncWrite for NamedPipeServer {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
            panic!("stub")
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            panic!("stub")
        }
        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            panic!("stub")
        }
    }

    impl AsRawHandle for NamedPipeServer {
        fn as_raw_handle(&self) -> RawHandle {
            panic!("stub")
        }
    }

    #[derive(Debug)]
    pub struct NamedPipeClient;

    impl io::AsyncRead for NamedPipeClient {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            panic!("stub")
        }
    }

    impl io::AsyncWrite for NamedPipeClient {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
            panic!("stub")
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            panic!("stub")
        }
        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            panic!("stub")
        }
    }

    impl AsRawHandle for NamedPipeClient {
        fn as_raw_handle(&self) -> RawHandle {
            panic!("stub")
        }
    }

    pub(super) fn new_server(
        name: &str,
        reject_remote_clients: bool,
        write: bool,
    ) -> io::Result<NamedPipeServer> {
        panic!("stub")
    }

    pub(super) fn new_client(name: &str, write: bool) -> io::Result<NamedPipeClient> {
        panic!("stub")
    }
}

fn genname() -> String {
    let procid = process::id();
    let random = rand::random::<usize>();

    format!(r"\\.\pipe\__tokio_anonymous_pipe0__.{}.{}", procid, random)
}

/// Asyncronous Pipe Read.
#[derive(Debug)]
pub enum AnonPipeRead {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

impl AnonPipeRead {
    async fn connect(&self) -> io::Result<()> {
        match self {
            Self::Server(inner) => inner.connect().await?,
            _ => panic!("not a server"),
        }
        Ok(())
    }
}

impl io::AsyncRead for AnonPipeRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Server(ref mut inner) => Pin::new(inner).poll_read(cx, buf),
            Self::Client(ref mut inner) => Pin::new(inner).poll_read(cx, buf),
        }
    }
}

impl IntoRawHandle for AnonPipeRead {
    fn into_raw_handle(self) -> RawHandle {
        let h = match &self {
            Self::Server(inner) => inner.as_raw_handle(),
            Self::Client(inner) => inner.as_raw_handle(),
        };
        mem::forget(self);
        h
    }
}

impl AsRawHandle for AnonPipeRead {
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            Self::Server(inner) => inner.as_raw_handle(),
            Self::Client(inner) => inner.as_raw_handle(),
        }
    }
}

/// Asyncronous Pipe Write.
#[derive(Debug)]
pub enum AnonPipeWrite {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

impl AnonPipeWrite {
    async fn connect(&self) -> io::Result<()> {
        match self {
            Self::Server(inner) => inner.connect().await?,
            _ => panic!("not a server"),
        }
        Ok(())
    }
}

impl io::AsyncWrite for AnonPipeWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            Self::Server(ref mut inner) => Pin::new(inner).poll_write(cx, buf),
            Self::Client(ref mut inner) => Pin::new(inner).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Self::Server(ref mut inner) => Pin::new(inner).poll_flush(cx),
            Self::Client(ref mut inner) => Pin::new(inner).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Self::Server(ref mut inner) => Pin::new(inner).poll_shutdown(cx),
            Self::Client(ref mut inner) => Pin::new(inner).poll_shutdown(cx),
        }
    }
}

impl IntoRawHandle for AnonPipeWrite {
    fn into_raw_handle(self) -> RawHandle {
        let h = match &self {
            Self::Server(inner) => inner.as_raw_handle(),
            Self::Client(inner) => inner.as_raw_handle(),
        };
        mem::forget(self);
        h
    }
}

impl AsRawHandle for AnonPipeWrite {
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            Self::Server(inner) => inner.as_raw_handle(),
            Self::Client(inner) => inner.as_raw_handle(),
        }
    }
}

#[derive(Debug)]
pub struct Connect<T>(T);

impl Connect<AnonPipeRead> {
    pub async fn connect(self) -> io::Result<AnonPipeRead> {
        self.0.connect().await?;
        Ok(self.0)
    }
}

impl Connect<AnonPipeWrite> {
    pub async fn connect(self) -> io::Result<AnonPipeWrite> {
        self.0.connect().await?;
        Ok(self.0)
    }
}

#[cfg(windows)]
fn new_server(name: &str, reject_remote_clients: bool, write: bool) -> io::Result<NamedPipeServer> {
    ServerOptions::new()
        .access_inbound(!write) // client to server
        .access_outbound(write) // server to client
        .first_pipe_instance(true)
        .reject_remote_clients(reject_remote_clients)
        .max_instances(1)
        .create(&name)
}

#[cfg(windows)]
fn new_client(name: &str, write: bool) -> io::Result<NamedPipeClient> {
    ClientOptions::new().read(!write).write(write).open(&name)
}

fn try_new_server(write: bool) -> io::Result<(String, NamedPipeServer)> {
    // https://www.rpi.edu/dept/cis/software/g77-mingw32/include/winerror.h
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_INVALID_PARAMETER: i32 = 87;

    let mut tries = 0;
    let mut reject_remote_clients = true;
    loop {
        tries += 1;
        let name = genname();

        let server = match new_server(&name, reject_remote_clients, write) {
            Ok(server) => server,
            Err(err) if tries < 10 => {
                match err.raw_os_error() {
                    Some(ERROR_ACCESS_DENIED) => continue,
                    Some(ERROR_INVALID_PARAMETER) if reject_remote_clients => {
                        // https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L101
                        reject_remote_clients = false;
                        tries -= 1;
                        continue;
                    }
                    _ => return Err(err),
                }
            }
            Err(err) => return Err(err),
        };
        return Ok((name, server));
    }
}

/// Open Anonynous Pipe Pair
pub async fn anon_pipe() -> io::Result<(AnonPipeRead, AnonPipeWrite)> {
    let (name, server) = try_new_server(false)?;
    let client = new_client(&name, true)?;

    server.connect().await?;

    let read = AnonPipeRead::Server(server);
    let write = AnonPipeWrite::Client(client);
    Ok((read, write))
}

/// Open Anonynous Pipe Pair
pub fn anon_pipe_we_read() -> io::Result<(Connect<AnonPipeRead>, AnonPipeWrite)> {
    let (name, server) = try_new_server(false)?;
    let client = new_client(&name, true)?;

    let read = Connect(AnonPipeRead::Server(server));
    let write = AnonPipeWrite::Client(client);
    Ok((read, write))
}

/// Open Anonynous Pipe Pair
pub fn anon_pipe_we_write() -> io::Result<(AnonPipeRead, Connect<AnonPipeWrite>)> {
    let (name, server) = try_new_server(true)?;
    let client = new_client(&name, false)?;

    let read = AnonPipeRead::Client(client);
    let write = Connect(AnonPipeWrite::Server(server));
    Ok((read, write))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test2() -> io::Result<()> {
        let (mut r, mut w) = anon_pipe().await?;

        w.write_all(b"Hello, World!").await?;
        let mut buf = vec![0; 13];
        let mut n = 0;
        while n < 13 {
            n += r.read(&mut buf[n..]).await?;
        }
        assert_eq!(&b"Hello, World!"[..], &buf);
        Ok(())
    }

    #[tokio::test]
    async fn test() {
        let (mut r, mut w) = anon_pipe().await.unwrap();

        let w_task = tokio::spawn(async move {
            for n in 0..=65535 {
                w.write_u32(n).await.unwrap();
            }
            //w.shutdown().await.unwrap();
        });

        let r_task = tokio::spawn(async move {
            let mut n = 0u32;
            let mut buf = [0; 4 * 128];
            while n < 65535 {
                r.read_exact(&mut buf).await.unwrap();
                for x in buf.chunks(4) {
                    assert_eq!(x, n.to_be_bytes());
                    n += 1;
                }
            }
        });
        tokio::try_join!(w_task, r_task).unwrap();
    }

    #[tokio::test]
    async fn test_write_after_shutdown() {
        let (r, mut w) = anon_pipe().await.unwrap();
        w.shutdown().await.unwrap();
        let result = w.write(b"ok").await;
        assert!(result.is_ok());

        drop(r)
    }
}
