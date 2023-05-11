.PHONY: all
all: potential.h libpotential.a

libpotential.a: src/*.rs
ifeq ($(ARCH_NAME),mac-x64)
	env RUSTFLAGS="-C target-cpu=nehalem" cargo build --release --target x86_64-apple-darwin
	cp target/x86_64-apple-darwin/release/libpotential.a .
else
$(error Unknown ARCH_NAME $(ARCH_NAME))
endif

potential.h: cbindgen.toml src/lib.rs
	cbindgen --config cbindgen.toml --crate potential --output potential.h

.PHONY: clean
clean:
	rm -f libpotential.a potential.h
	cargo clean
