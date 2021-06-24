# tokio-anon-pipe

Asynchronous anonymous pipe for Windows.

inspired by
https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L48

## Example

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (mut r, mut w) = tokio_anon_pipe::anon_pipe().await?;

    w.write_all(b"HELLO, WORLD!").await?;

    let mut buf = [0; 16];
    let len = r.read(&mut buf[..]).await?;

    assert_eq!(&buf[..len], &b"HELLO, WORLD!"[..]);
    Ok(())
}
```
