# Potential for VCV Rack
Potential is a collection of modules for [VCV Rack](https://vcvrack.com/). These
modules attempt to fill some niches not covered by existing plugins.

This is also a testing ground for implementing Rack plugins in Rust, to the
extent that it's possible to do that.

## Modules
* Breaker: Watch signal levels as they pass through the module, and trip a "breaker" circuit, muting the signal, if a threshold is exceeded.
* MagSign: Split out the magnitude (absolute value) and sign (+1.0 or -1.0) of a bipolar input. Also, put them back together again.
* PolyShuffle: Shuffle (randomize) the order of polyphonic channels.

## Status
At the time of writing, this is fairly early in the lifecycle of this plugin.
There are no releases at the moment, so you will need to build from source.

Since we're relying on the VCV Rack SDK's Makefile framework, we have a separate
target to build the Rust library first before proceeding with the built-in SDK
targets. Either run `make rustlib` before running `make`, or run `make full` to
semi-automatically run one and then the other.

The panel graphics are very rudimentary. 

The plugin is also not in the VCV Rack library. Once there are automatic builds
in this repository, we will see what the road looks like to submitting it.
