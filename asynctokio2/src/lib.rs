#![no_main]

mod error;

use std::os::fd::FromRawFd;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
};

const READ_BUFFER_SIZE: usize = 1024; // 1KB is shorter than common MTU but longer than common TCP MSS

#[export_name = "_worker"]
pub fn _worker(fd: i32) -> i32 {
    let stdstream = unsafe { std::net::TcpStream::from_raw_fd(fd) };
    stdstream
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    match tokio_worker(stdstream) {
        Ok(_) => 0,
        Err(e) => {
            println!("tokio_worker failed: {}", e);
            return -1;
        }
    };

    return error::Error::None.i32();
}

#[tokio::main(flavor = "current_thread")]
async fn tokio_worker(conn: std::net::TcpStream) -> std::io::Result<()> {
    let mut conn: tokio::net::TcpStream =
        tokio::net::TcpStream::from_std(conn).expect("Failed to convert to tokio stream");

    println!("tokio_worker: conn = {:?}", conn);

    loop{
        // Wait for the socket to be readable
        conn.readable().await?;

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut buf = [0; READ_BUFFER_SIZE];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match conn.try_read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                println!("read {} bytes", n);
                if let Err(e) = conn.write_all(&buf[0..n]).await {
                    println!("Error writing to conn: {:?}", e);
                    return Err(e);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("WouldBlock");
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
