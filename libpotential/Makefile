.PHONY: all
all: libpotential.a ffi.rs.h

# One of: lin-x64 mac-arm64 mac-x64 win-x64
ifdef TARGET_ARCH

ifeq ($(TARGET_ARCH),lin-x64)
CARGO_ENV_VARS := RUSTFLAGS="-C target-cpu=nehalem"
RUST_TARGET_NAME := x86_64-unknown-linux-gnu

else ifeq ($(TARGET_ARCH),mac-arm64)
# This may not generate exactly the same instruction set as what Rack's
# C++ "-march=armv8-a+fp+simd" does, but I think it's reasonably safe.
CARGO_ENV_VARS := RUSTFLAGS="-C target-cpu=apple-m1"
RUST_TARGET_NAME := aarch64-apple-darwin

else ifeq ($(TARGET_ARCH),mac-x64)
CARGO_ENV_VARS := RUSTFLAGS="-C target-cpu=nehalem"
RUST_TARGET_NAME := x86_64-apple-darwin

else ifeq ($(TARGET_ARCH),win-x64)
CARGO_ENV_VARS := RUSTFLAGS="-C target-cpu=nehalem"
RUST_TARGET_NAME := x86_64-pc-windows-gnu

else
$(error Unknown TARGET_ARCH $(TARGET_ARCH))

endif

CARGO_ARGS := --target $(RUST_TARGET_NAME)

else
CARGO_ENV_VARS := RUSTFLAGS="-C target-cpu=native"
endif

libpotential.a ffi.rs.h: src/*.rs
	env $(CARGO_ENV_VARS) cargo build --release $(CARGO_ARGS)
	cp target/$(RUST_TARGET_NAME)/release/libpotential.a .
	cp target/$(RUST_TARGET_NAME)/cxxbridge/potential/src/ffi.rs.h .

.PHONY: clean
clean:
	rm -f libpotential.a ffi.rs.h
	cargo clean
