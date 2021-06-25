# tokio-anon-pipe

Asynchronous anonymous pipe for Windows.

inspired by
<https://github.com/rust-lang/rust/blob/456a03227e3c81a51631f87ec80cac301e5fa6d7/library/std/src/sys/windows/pipe.rs#L48>

> Note that we specifically do *not* use `CreatePipe` here because
> unfortunately the anonymous pipes returned do not support overlapped
> operations. Instead, we create a "hopefully unique" name and create a
> named pipe which has overlapped operations enabled.

## Supported platform

`x86_64-pc-windows-msvc` only

## Example

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let (mut r, mut w) = tokio_anon_pipe::anon_pipe().await?;

    w.write_all(b"HELLO, WORLD!").await?;

    let mut buf = [0; 16];
    let len = r.read(&mut buf[..]).await?;

    assert_eq!(&buf[..len], &b"HELLO, WORLD!"[..]);
    Ok(())
}
```

License: MIT/Apache-2.0
