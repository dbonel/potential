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

The panel graphics are very rudimentary. 

The plugin is also not in the VCV Rack library. Once there are automatic builds
in this repository, we will see what the road looks like to submitting it.

## Building
In addition to the usual VCV Rack [plugin development prerequisites](https://vcvrack.com/manual/PluginDevelopmentTutorial),
you will also need a Rust toolchain. [Rustup](https://rustup.rs/) is probably
the best method to get set up.

If you'd like to force a specific target architecture (e.g. if you're on an
Apple Silicon Mac but would like to compile for x64 to match your Rack install),
you can do that by setting the `TARGET_ARCH` variable. This variable should
match the format Rack uses for architectures, so one of `lin-x64`, `mac-arm64`,
`mac-x64`, or `win-x64`.

If you do not set a `TARGET_ARCH` variable, the Rust library will be optimized
for the CPU it's built on, so anything that's intended to be portable should
probably set an explicit `TARGET_ARCH`.

For example:
```console
$ make TARGET_ARCH=mac-x64
```

We also expose the usual `make dist` and `make install` targets from the Rack
SDK.
