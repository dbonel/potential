#pragma once

#include <engine/Port.hpp>
#include <ffi.rs.h>

inline rustlib::Port *ffi_port(rack::engine::Port *p) {
    return reinterpret_cast<rustlib::Port *>(p);
}
