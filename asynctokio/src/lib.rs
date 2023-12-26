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
        Ok(_) => error::Error::None.i32(),
        Err(e) => {
            println!("tokio_worker failed: {}", e);
            return error::Error::FailedIO.i32();
        }
    };

    return error::Error::None.i32();
}

#[tokio::main(flavor = "current_thread")]
async fn tokio_worker(conn: std::net::TcpStream) -> std::io::Result<()> {
    let mut conn: tokio::net::TcpStream =
        tokio::net::TcpStream::from_std(conn).expect("Failed to convert to tokio stream");

    println!("tokio_worker: conn = {:?}", conn);

    let mut rd_buf = vec![0; READ_BUFFER_SIZE];

    loop {
        tokio::select! {
            result = conn.read(&mut rd_buf) => {
                // println!("dst.read() result = {:?}", result);
                match result {
                    Ok(0) => break, // End of stream
                    Ok(n) => {
                        println!("read {} bytes", n);
                        if let Err(e) = conn.write_all(&rd_buf[0..n]).await {
                            println!("Error writing to conn: {:?}", e);
                            return Err(e);
                        }
                    }
                    Err(e) => {
                        println!("Error reading from conn: {:?}", e);
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}
