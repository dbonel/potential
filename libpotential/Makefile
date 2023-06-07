.PHONY: all
all: libpotential.a ffi.rs.h

libpotential.a ffi.rs.h: src/*.rs
ifeq ($(ARCH_NAME),mac-x64)
	env RUSTFLAGS="-C target-cpu=nehalem" cargo build --release --target x86_64-apple-darwin
	cp target/x86_64-apple-darwin/release/libpotential.a .
	cp target/x86_64-apple-darwin/cxxbridge/potential/src/ffi.rs.h .
endif
ifeq ($(ARCH_NAME),lin-x64)
	env RUSTFLAGS="-C target-cpu=nehalem" cargo build --release --target x86_64-unknown-linux-gnu
	cp target/x86_64-unknown-linux-gnu/release/libpotential.a .
	cp target/x86_64-unknown-linux-gnu/cxxbridge/potential/src/ffi.rs.h .
endif
ifeq ($(ARCH_NAME),win-x64)
	env RUSTFLAGS="-C target-cpu=nehalem" cargo build --release --target x86_64-pc-windows-gnu
	cp target/x86_64-pc-windows-gnu/release/libpotential.a .
	cp target/x86_64-pc-windows-gnu/cxxbridge/potential/src/ffi.rs.h .
endif

.PHONY: clean
clean:
	rm -f libpotential.a ffi.rs.h
	cargo clean
