// Error is a enum in i32
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error {
    None = 0,
    Unknown = -1,         // general error
    InvalidArgument = -2, // invalid argument supplied to func call
    InvalidConfig = -3,   // config file provided is invalid
    InvalidFd = -4,       // invalid file descriptor provided
    InvalidFunction = -5, // invalid function called
    DoubleInit = -6,      // initializing twice
    FailedIO = -7,        // Failing an I/O operation
    NotInitialized = -8,  // not initialized
}

impl Error {
    pub fn i32(&self) -> i32 {
        *self as i32
    }
}
