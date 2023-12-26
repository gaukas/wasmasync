#![no_main]

mod error;

use std::io::{Read, Write};
use std::os::fd::FromRawFd;

const READ_BUFFER_SIZE: usize = 1024; // 1KB is shorter than common MTU but longer than common TCP MSS

#[export_name = "_worker"]
pub fn _worker(fd: i32) -> i32 {
    let mut stdstream = unsafe { std::net::TcpStream::from_raw_fd(fd) };
    stdstream
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    // in a loop, read from fd, write to fd
    let mut buf = vec![0; READ_BUFFER_SIZE];
    loop {
        match stdstream.read(&mut buf) {
            Ok(0) => break, // End of stream
            Ok(n) => {
                if let Err(e) = stdstream.write(&buf[..n]) {
                    println!("Error writing to fd: {:?}", e);
                    return error::Error::FailedIO.i32();
                }
                println!("written {} bytes", n)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("WouldBlock... retrying."); // this is good, will be printed for many times
                continue; // retry
            }
            Err(e) => {
                println!("Error reading from fd: {:?}", e);
                return error::Error::FailedIO.i32();
            }
        }
    }

    return error::Error::None.i32();
}
