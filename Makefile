# If RACK_DIR is not defined when calling the Makefile, default to two directories above
RACK_DIR ?= ../..

# FLAGS will be passed to both the C and C++ compiler
FLAGS += -I./libpotential
CFLAGS +=
CXXFLAGS +=

# Careful about linking to shared libraries, since you can't assume much about the user's environment and library search path.
# Static libraries are fine, but they should be added to this plugin's build system.
LDFLAGS += libpotential/libpotential.a

# Add .cpp files to the build
SOURCES += $(wildcard src/*.cpp)

# Add files to the ZIP package when running `make dist`
# The compiled plugin and "plugin.json" are automatically added.
DISTRIBUTABLES += res
DISTRIBUTABLES += $(wildcard LICENSE*)
DISTRIBUTABLES += $(wildcard presets)

# Include the Rack plugin Makefile framework
include $(RACK_DIR)/plugin.mk

libpotential/libpotential.a:
	make -C libpotential libpotential.a

libpotential/potential.h:
	make -C libpotential potential.h

.PHONY: rustlib
rustlib: libpotential/libpotential.a libpotential/potential.h

.PHONY: full
full: rustlib all

.PHONY: fullclean
fullclean: clean
	make -C libpotential clean
