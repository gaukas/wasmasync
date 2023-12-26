CARGO=cargo 
TARGET=wasm32-wasi
TARGETDIR=./target/wasm32-wasi/debug
SUFFIX=wasm

all: clean
	$(CARGO) build --target $(TARGET)
	mv $(TARGETDIR)/*.$(SUFFIX) ./wasm/

asyncrs:
	cd $@; $(CARGO) build --target $(TARGET);
	mv $(TARGETDIR)/$@.$(SUFFIX) ./wasm/

asynctokio:
	cd $@; $(CARGO) build --target $(TARGET);
	mv $(TARGETDIR)/$@.$(SUFFIX) ./wasm/

asynctokio2:
	cd $@; $(CARGO) build --target $(TARGET);
	mv $(TARGETDIR)/$@.$(SUFFIX) ./wasm/

syncrs:
	cd $@; $(CARGO) build --target $(TARGET);
	mv $(TARGETDIR)/$@.$(SUFFIX) ./wasm/

clean:
	rm -rf ./wasm/*

.PHONY: all asyncrs asynctokio asynctokio2 syncrs clean