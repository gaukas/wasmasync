(module
    ;; Import the required fd_write WASI function which will write the given io vectors to stdout
    ;; The function signature for fd_write is:
    ;; (File Descriptor, *iovs, iovs_len, nwritten) -> Returns number of bytes written
    (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

    ;; Import the required fd_read WASI function which will read from stdin into the given io vectors
    ;; The function signature for fd_read is:
    ;; (File Descriptor, *iovs, iovs_len, nread) -> Returns number of bytes read
    (import "wasi_snapshot_preview1" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))

    (memory 1)
    (export "memory" (memory 0))

    ;; Create a data segment to store the string 'hello world\n' at offset 8 in the linear memory, 
    ;; and its length as a 32-bit integer at offset 4 in the linear memory.
    ;; We will overwrite the 'hello world\n' string with the string we read later on.
    (data (i32.const 4) "") ;; 4 is the offset into the linear memory
    (data (i32.const 8) "hello world\n") ;; 8 is the offset into the linear memory

    ;; read
    ;;
    ;; This function will read a string from stdin and store it in the linear memory at offset 8
    ;; and its length as a 32-bit integer at offset 4.
    (func $read (export "_read") (param $fd i32) (result i32)
        ;; Creating a new io vector within linear memory
        (i32.store (i32.const 0) (i32.const 8))  ;; iov.iov_base - This is a pointer to the start of the buffer.
        (i32.store (i32.const 4) (i32.const 64))  ;; iov.iov_len - The maximum length of the buffer, which is 72-8 = 64 bytes

        (call $fd_read
            (local.get $fd) ;; file_descriptor - 0 for stdin
            (i32.const 0) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
            (i32.const 1) ;; iovs_len - We're reading 1 string stored in an iov - so one.
            (i32.const 4) ;; nread - A place in memory to store the number of bytes read
        )
        
        drop ;; drop the return value from fd_read		
        (i32.load (i32.const 4)) ;; Return the number of bytes read from the top of the stack
    )

    ;; write
    (func $write (export "_write") (param $fd i32) (result i32)
        ;; Creating a new io vector within linear memory
        (i32.store (i32.const 0) (i32.const 8))  ;; iov.iov_base - This is a pointer to the start of the 'hello world\n' string
        ;; (i32.store (i32.const 4) (i32.const 12))  ;; iov.iov_len - The length of the 'hello world\n' string

        (call $fd_write
            (local.get $fd) ;; file_descriptor - 1 for stdout
            (i32.const 0) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
            (i32.const 1) ;; iovs_len - We're printing 1 string stored in an iov - so one.
            (i32.const 72) ;; nwritten - A place in memory to store the number of bytes written
        )
        
        drop ;; drop the return value from fd_write		
        (i32.load (i32.const 72)) ;; Return the number of bytes written from the top of the stack
    )

    ;; worker
    ;;
    ;; call read, then write, in a loop
    (func $worker (export "_worker") (param $fd i32) (result i32)
        ;; variable len i32
        (local $len i32)
        
        (loop $loop
            ;; Read from fd
            (call $read (local.get $fd))
            
            ;; Write to fd
            (call $write (local.get $fd))
            (local.set $len) ;; Store the length of the string written to stdout

            ;; If the length of the string read from stdin is greater than 0, then loop
            (local.get $len)
            (i32.const 0)
            (i32.ne)
            (br_if $loop)
            drop
        )

        local.get $len
    )


    (func $main (export "_start")
        ;; Do nothing
    )
)