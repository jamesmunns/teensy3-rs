# Rust on Teensy3
This repo contains raw rust bindings to Teensy3's c/c++ core library, generated with rust-bindgen. Compatible with Teensy 3.0, 3.1, 3.2, 3.5, and 3.6. Tested on only 3.6.

## Notes about this fork
This repo is fork of otaku's [teensy3-rs](https://github.com/otaku/teensy3-rs), which is fork of jamesmunns's [teensy3-rs](https://github.com/jamesmunns/teensy3-rs). The main changes in this fork are
* Continuing previous work and getting it all to *work on my machine* 
* Finishing up otaku's work on supporting all models 3.0, 3.1, 3.2, 3.5, and 3.6
* Updated teensy3 libraries and updated rust compatibility
* Hardware floating point support is available for 3.5 and 3.6
* Updated compatibility with newer rust nightly compiler
* Many modifcations to `build.rs`, trying to minimize configuration needed


## Getting Started
This crate is meant to be used with template project [teensy3-rs-demo](https://github.com/tolvanea/teensy3-rs-demo). Cross compiling is pain, so please use that template. It uses [cross](https://github.com/rust-embedded/cross) and runs cross-compilation in docker container, so correct all toolchain dependencies and libraries are automatically fetched. It *just works*. 

If you plan not to use that template, then be prepared to fight against c/c++ code compilation and binding generation. You should at least copy template's linker configurations from `.cargo/config`. The build script assumes that c standard libraries are found from conventional paths of linux environment. Without the docker environment, you probably need to configure all library paths manually in `build.rs`. Also, library paths varies between different versions.

## Package layout
* `teensy3-sys` - This crate contains the C/C++ code and the Rust bindings against them. All items are generally unsafe, and not idiomatic rust. `teensy3-sys` is re-exported as `teensy3::bindings`.
* `teensy3` - This crate contains few ergonomic wrappers around `teensy3-sys` components, as well as any pure rust reimplementations of other components. However, almost all bindings are just raw output from rust-bindgen.

## Dependencies
* A somewhat current Nightly Build of Rust (currently tested on `rustc 1.47.0-nightly`)
* [teensy-loader-cli](https://www.pjrc.com/teensy/loader_cli.html)
  for flashing your program onto hardware.
* If you use template [teensy3-rs-demo](https://github.com/tolvanea/teensy3-rs-demo), then docker and cargo cross is required. 
* If template is not used, then numerous dependencies need to be installed manually. On ubuntu, most of them can be probably installed with:
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

## Adding bindings to new library
If the desired library is not included in this repo, then it may be relatively easy to include it. Git clone this repo, place library to root folder and add library name to `bindings.h` and `build.rs`. Direct example can be taken from `core` and `SPI`. However, there may be compilation and bindgen errors, so some extra configuration is probably needed. 


## Thanks, Citiations
This code is nearly entirely thanks to these resources:

* [PJRC's Teensyduino libraries](https://github.com/PaulStoffregen/cores) for the Teensy3, which are used as bindings.
* [Simon's teensy3-clock repo](https://github.com/SimonSapin/teensy-clock) for the rust main, build scripts, bindgen knowledge, et. al.
* [rust-bindgen](https://github.com/servo/rust-bindgen)

## License

Rust contributions are licensed under the MIT License.

**Please Note:** ASM, C, C++, and Linker Components of the `teensy3-sys` crate (a dependency of the `teensy3` crate) contain components licensed under the MIT License, PJRC's modified MIT License, and the LGPL v2.1. Please refer to individual components for more details.
