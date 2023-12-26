# WASM Async driver/examples

This is a simple repo for async WebAssembly operations under strictly wasi preview1. 

On the runtime side, we use [wazero](https://github.com/tetratelabs/wazero) to instantiate and run the wasm module: a TCP connection (`*net.TCPConn`) will be 
created and pushed into the WebAssembly instance as a file descriptor. 

The WebAssembly instance will perform read/write operations on the file descriptor on a BLOCKING thread, but the operation performed on the file descriptor is expected to be non-blocking, i.e., we can add reasonable concurrency to the WebAssembly module to operate on multiple file descriptors at once.

## Usage

```
go run ./ -wasm ./path/to/file.wasm
```