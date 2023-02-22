# Rust on Teensy 3
This repository generates rust binding for Teensy's c/c++ libraries. Teensy 3 is arduino-like ARM based microcontroller. Compatible Teensy models are 3.0, 3.1, 3.2, 3.5, and 3.6. Tested only on Teensy 3.6. 

## Notes about this fork
This repo is fork of otaku's [teensy3-rs](https://github.com/otaku/teensy3-rs), which is fork of jamesmunns's [teensy3-rs](https://github.com/jamesmunns/teensy3-rs). The main changes in this fork are
* Continuing previous work and getting it all to work on docker 
* Continuing otaku's work on supporting all models 3.0, 3.1, 3.2, 3.5, and 3.6
* Updated teensy3 libraries and updated rust 1.47 compatibility
* Hardware floating point support is now available for Teensy 3.5 and 3.6


## Getting Started
This crate is meant to be used with template project [teensy3-rs-demo](https://github.com/tolvanea/teensy3-rs-demo). The template is highly recommended, because there is high change to make mistake with c/c++ source code compilation, binding generation or cross compilation. By default, the template uses [Cross](https://github.com/rust-embedded/cross) instead of Cargo. Cross runs cross-compilation in docker container, and all toolchain dependencies and libraries are automatically fetched. It *just works*. 

If Cross is not used, then note that the build script assumes conventional paths of linux environment. There is probably need to configure library paths manually in `build.rs`. Different library version tend to have different paths, which is one reason why Cross is endorsed.

## Package layout
* `teensy3-sys` - This crate contains the C/C++ code and the Rust bindings against them. All items are generally unsafe, and not idiomatic rust. `teensy3-sys` is re-exported as `teensy3::bindings`.
* `teensy3` - This crate contains few ergonomic wrappers around `teensy3-sys` components. However, there is not many safe wrappers.

## Dependencies
* A somewhat current version of Rust (tested on `rustc 1.47.0-stable`)
* [teensy-loader-cli](https://www.pjrc.com/teensy/loader_cli.html)
  for flashing programs into hardware.
* Cross and Docker are highly recommended. See more in template [teensy3-rs-demo](https://github.com/tolvanea/teensy3-rs-demo).
* If cross is not used, then numerous dependencies need to be installed manually. On ubuntu, most of them can be probably installed with:
```
apt-get install \
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi \
    libnewlib-dev \
    libstdc++-arm-none-eabi-newlib \
    clang \
    libclang-8-dev \
    gcc-multilib
```

## Adding bindings to some new C/C++ library
If the desired Teensy library is not covered by this repo, it may be relatively easy to generate bindigs for it. Direct example can be taken from `core` or `SPI` libraries: Place library to root folder of this crate and add its name to `bindings.h` and `build.rs`. However, there may be compilation errors, so some extra configuration is probably needed. 


## Thanks, Citiations
This code is nearly entirely thanks to these resources:

* [PJRC's Teensyduino libraries](https://github.com/PaulStoffregen/cores) for the Teensy3, which are used as bindings.
* [Simon's teensy3-clock repo](https://github.com/SimonSapin/teensy-clock) for the rust main, build scripts, bindgen knowledge, et. al.
* [rust-bindgen](https://github.com/servo/rust-bindgen)

## License

Rust contributions are licensed under the MIT License.

**Please Note:** ASM, C, C++, and Linker Components of the `teensy3-sys` crate (a dependency of the `teensy3` crate) contain components licensed under the MIT License, PJRC's modified MIT License, and the LGPL v2.1. Please refer to individual components for more details.
