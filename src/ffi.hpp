#pragma once

#include <engine/Module.hpp>
#include <engine/Port.hpp>
#include <ffi.rs.h>

inline rustlib::Port *ffi_port(rack::engine::Port *p) {
    return reinterpret_cast<rustlib::Port *>(p);
}

inline void configure_from_info(rack::engine::Module *rack_module,
                                rustlib::ModuleConfigInfo *config) {
    size_t in_port_count = config->get_input_port_count();
    size_t out_port_count = config->get_output_port_count();
    size_t param_count = 0;
    size_t light_count = 0;
    rack_module->config(param_count, in_port_count, out_port_count,
                        light_count);

    for (size_t i = 0; i < in_port_count; ++i) {
        auto name = std::string(config->get_input_port_name(i));
        rack_module->configInput(i, name);
    }

    for (size_t i = 0; i < out_port_count; ++i) {
        auto name = std::string(config->get_output_port_name(i));
        rack_module->configOutput(i, name);
    }
}
