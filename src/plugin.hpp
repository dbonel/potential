#pragma once
#include <rack.hpp>

#include "ffi.hpp"

using namespace rack;

// Declare the Plugin, defined in plugin.cpp
extern Plugin *pluginInstance;

// Declare each Model, defined in each module source file
extern Model *modelMagSign;
extern Model *modelBreaker;
extern Model *modelPolyShuffle;
