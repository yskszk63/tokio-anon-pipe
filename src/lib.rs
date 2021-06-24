//! # tokio-anon-pipe
//!
//! Asynchronous anonymous pipe for windows.
//!
//! inspired by
//! https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L48
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

    pub(super) fn new_server(name: &str) -> io::Result<NamedPipeServer> {
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

#[derive(Debug)]
pub struct AnonPipWrite {
    inner: NamedPipeClient,
}

impl io::AsyncWrite for AnonPipWrite {
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
fn new_server(name: &str) -> io::Result<NamedPipeServer> {
    ServerOptions::new()
        .access_inbound(true) // client to server
        .access_outbound(false) // server to client
        .first_pipe_instance(true)
        .reject_remote_clients(true)
        .max_instances(1)
        .create(&name)
}

#[cfg(windows)]
fn new_client(name: &str) -> io::Result<NamedPipeClient> {
    ClientOptions::new().read(false).write(true).open(&name)
}

pub async fn anon_pipe() -> io::Result<(AnonPipeRead, AnonPipWrite)> {
    // TODO retry
    let name = genname();

    let server = new_server(&name)?;
    let client = new_client(&name)?;

    server.connect().await?;

    let read = AnonPipeRead { inner: server };
    let write = AnonPipWrite { inner: client };
    Ok((read, write))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() -> io::Result<()> {
        let (mut r, mut w) = anon_pipe().await?;
        println!("{:?} {:?}", r, w);
        Ok(())
    }
}
