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

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
