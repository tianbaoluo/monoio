//! An echo example.
//!
//! Run the example and `nc 127.0.0.1 50002` in another shell.
//! All your input will be echoed out.

use monoio::{
    io::{AsyncReadRent, AsyncWriteRentExt},
    net::{TcpListener, TcpStream},
};

#[monoio::main(driver = "fusion")]
async fn main() {
    let mut buf = vec![0; 1024];

    let canceler = monoio::io::Canceller::new();
    let handle = canceler.handle();

    let mut timer = std::pin::pin!(monoio::time::sleep(std::time::Duration::from_millis(100)));
    let mut recv = std::pin::pin!(conn.cancelable_read(buf, handle));

    monoio::select! {
    _ = &mut timer => {
        canceler.cancel();
        let (res, _buf) = recv.await;
        if matches!(res, Err(e) if e.raw_os_error() == Some(125)) {
            // Canceled success
            buf = _buf;
            todo!()
        }
        // Canceled but executed
        // Process res and buf
        todo!()
    },
    r = &mut recv => {
        let (res, _buf) = r;
        // Process res and buf
        todo!()
    }
}
}
