package main

import (
	"bytes"
	"context"
	"crypto/rand"
	"flag"
	"log"
	"math/big"
	"net"
	"os"
	"sync"
	"syscall"
	"time"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

var wasmPath = flag.String("wasm", "", "path to the wasm file")

func main() {
	flag.Parse()

	var wasm []byte
	var err error

	if *wasmPath != "" {
		log.Printf("reading wasm file: %s", *wasmPath)
		// read the wasm file
		wasm, err = os.ReadFile(*wasmPath)
	} else {
		log.Fatal("usage: ./wasmasync -wasm <path to wasm file>")
		// log.Printf("no -wasm given, using default wat file")
		// wasm, err = wasmtime.Wat2Wasm(string(wat))
	}
	if err != nil {
		panic(err)
	}

	// Choose the context to use for function calls.
	ctx := context.Background()

	// Create a new WebAssembly Runtime.
	r := wazero.NewRuntime(ctx)
	defer r.Close(ctx) // This closes everything this Runtime created.

	wasi_snapshot_preview1.MustInstantiate(ctx, r)

	mConfig := wazero.NewModuleConfig()
	mConfig = mConfig.WithStdout(os.Stdout)
	mConfig = mConfig.WithStderr(os.Stderr)

	write, err := r.InstantiateWithConfig(ctx, wasm, mConfig)
	if err != nil {
		panic(err)
	}

	if write == nil {
		panic("write is nil")
	}

	// create a TCP Conn pair
	lis, err := net.ListenTCP("tcp", nil)
	if err != nil {
		panic(err)
	}
	defer lis.Close()

	var lisConn *net.TCPConn
	var lisWg sync.WaitGroup
	lisWg.Add(1)
	go func() {
		defer lisWg.Done()
		lisConn, err = lis.AcceptTCP()
		if err != nil {
			panic(err)
		}
	}()

	dialConn, err := net.DialTCP("tcp", nil, lis.Addr().(*net.TCPAddr))
	if err != nil {
		panic(err)
	}

	lisWg.Wait()

	// set non-blocking for lisConn
	lisConnRaw, err := lisConn.SyscallConn()
	if err != nil {
		panic(err)
	}

	err = lisConnRaw.Control(func(fd uintptr) {
		err = syscall.SetNonblock(int(fd), true) // unix only, on Windows use syscall.Handle instead.
		if err != nil {
			panic(err)
		}
	})
	if err != nil {
		panic(err)
	}

	fd, ok := write.InsertTCPConn(lisConn)
	if !ok {
		panic("failed to insert TCPConn")
	}

	// first goroutine:
	// generate random data up to 64 bytes
	// write to dialConn
	// read from dialConn
	// compare the result
	go func() {
		defer dialConn.Close()
		defer lisConn.Close()

		var wrBuf []byte = make([]byte, 64)
		var rdBuf []byte = make([]byte, 64)
		for {
			_, err := rand.Read(wrBuf)
			if err != nil {
				panic(err)
			}

			randN, err := rand.Int(rand.Reader, big.NewInt(64-16))
			if err != nil {
				panic(err)
			}

			nWr, err := dialConn.Write(wrBuf[:randN.Uint64()+16])
			if err != nil {
				panic(err)
			}

			log.Printf("wrote %d bytes\n", nWr)

			// then read from dialConn
			nRd, err := dialConn.Read(rdBuf)
			if err != nil {
				panic(err)
			}

			if nWr != nRd {
				panic("nWr != nRd")
			}

			if !bytes.Equal(wrBuf[:nWr], rdBuf[:nRd]) {
				panic("buf != rdBuf")
			}

			log.Printf("read %d bytes\n", nRd)

			time.Sleep(1 * time.Second)
		}
	}()

	// second goroutine:
	// blocking on worker function
	go func() {
		// call the function
		results, err := write.ExportedFunction("_worker").Call(ctx, uint64(fd))
		if err != nil {
			log.Panicln(err)
		}

		if len(results) != 1 {
			panic("unexpected number of results")
		}

		log.Println("worker exited")
		os.Exit(0)
	}()

	select {}
}
