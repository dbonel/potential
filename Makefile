all: plugin

# TARGET_ARCH, if set, should be in the Rack format, so one of:
# lin-x64 mac-arm64 mac-x64 win-x64
ifdef TARGET_ARCH
# The libpotential Makefile will use this.
export TARGET_ARCH

ifeq ($(TARGET_ARCH),lin-x64)
CROSS_COMPILE := x86_64-darwin

else ifeq ($(TARGET_ARCH),mac-arm64)
CROSS_COMPILE := arm64-darwin

else ifeq ($(TARGET_ARCH),mac-x64)
CROSS_COMPILE := x86_64-darwin

else ifeq ($(TARGET_ARCH),win-x64)
CROSS_COMPILE := x86_64-mingw32

else
$(error Unknown TARGET_ARCH $(TARGET_ARCH))

endif

# The VCV SDK Makefile framework will need this.
export CROSS_COMPILE

endif

libpotential/libpotential.a libpotential/ffi.rs.h: libpotential/src/*.rs
	$(MAKE) -C libpotential libpotential.a ffi.rs.h

.PHONY: plugin
plugin: res/* src/*.cpp src/*.hpp libpotential/libpotential.a libpotential/ffi.rs.h
	$(MAKE) -f Makefile.racksdk

.PHONY: rustlib
rustlib: libpotential/libpotential.a libpotential/ffi.rs.h

.PHONY: clean
clean:
	$(MAKE) -C libpotential clean
	$(MAKE) -f Makefile.racksdk clean

dist: plugin
	$(MAKE) -f Makefile.racksdk dist

install: plugin
	$(MAKE) -f Makefile.racksdk install
