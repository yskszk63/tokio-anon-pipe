//! # tokio-anon-pipe
//!
//! Asynchronous anonymous pipe for windows.
//!
//! inspired by
//! https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L48
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, RawHandle};
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

    #[derive(Debug)]
    pub struct NamedPipeServer;

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

    #[derive(Debug)]
    pub struct NamedPipeClient;

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

    pub(super) fn new_server(
        name: &str,
        reject_remote_clients: bool,
    ) -> io::Result<NamedPipeServer> {
        panic!("stub")
    }

    pub(super) fn new_client(name: &str) -> io::Result<NamedPipeClient> {
        panic!("stub")
    }
}

fn genname() -> String {
    let procid = process::id();
    let random = rand::random::<usize>();

    format!(r"\\.\pipe\__tokio_anonymous_pipe0__.{}.{}", procid, random)
}

#[derive(Debug)]
pub struct AnonPipeRead {
    inner: NamedPipeServer,
}

impl io::AsyncRead for AnonPipeRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

#[cfg(windows)]
impl AsRawHandle for AnonPipeRead {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_raw_handle()
    }
}

#[derive(Debug)]
pub struct AnonPipeWrite {
    inner: NamedPipeClient,
}

impl io::AsyncWrite for AnonPipeWrite {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(windows)]
impl AsRawHandle for AnonPipeWrite {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_raw_handle()
    }
}

#[cfg(windows)]
fn new_server(name: &str, reject_remote_clients: bool) -> io::Result<NamedPipeServer> {
    ServerOptions::new()
        .access_inbound(true) // client to server
        .access_outbound(false) // server to client
        .first_pipe_instance(true)
        .reject_remote_clients(reject_remote_clients)
        .max_instances(1)
        .create(&name)
}

#[cfg(windows)]
fn new_client(name: &str) -> io::Result<NamedPipeClient> {
    ClientOptions::new().read(false).write(true).open(&name)
}

pub async fn anon_pipe() -> io::Result<(AnonPipeRead, AnonPipeWrite)> {
    // https://www.rpi.edu/dept/cis/software/g77-mingw32/include/winerror.h
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_INVALID_PARAMETER: i32 = 87;

    let mut tries = 0;
    let mut reject_remote_clients = true;
    loop {
        tries += 1;
        let name = genname();

        let server = match new_server(&name, reject_remote_clients) {
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
        let client = new_client(&name)?;

        server.connect().await?;

        let read = AnonPipeRead { inner: server };
        let write = AnonPipeWrite { inner: client };
        return Ok((read, write));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test() -> io::Result<()> {
        let (mut r, mut w) = anon_pipe().await?;

        w.write_all(b"Hello, World!").await?;
        let mut buf = vec![];
        while buf.len() < "Hello, World!".len() {
            r.read(&mut buf).await?;
        }
        assert_eq!(&b"Hello, World!"[..], &buf);
        Ok(())
    }
}
