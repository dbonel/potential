.PHONY: all
all: libpotential.a ffi.rs.h

libpotential.a ffi.rs.h: src/*.rs
ifeq ($(ARCH_NAME),mac-x64)
	env RUSTFLAGS="-C target-cpu=nehalem" cargo build --release --target x86_64-apple-darwin
	cp target/x86_64-apple-darwin/release/libpotential.a .
	cp target/x86_64-apple-darwin/cxxbridge/potential/src/ffi.rs.h .
else
$(error Unknown ARCH_NAME $(ARCH_NAME))
endif

.PHONY: clean
clean:
	rm -f libpotential.a ffi.rs.h
	cargo clean
