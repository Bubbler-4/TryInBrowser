# TryInBrowser

Online interpreter that works even if you go offline

## Why Try In Browser?

Most online interpreters require a server to run the submitted programs. And making different runs not interfere with each other is difficult.

Try In Browser attempts to solve this problem by moving all the interpreters into the browser. No need to isolate runs. No need to run a dedicated server;
a simple file server (such as GitHub Pages) is enough. An added benefit is that, once everything is loaded, you can disconnect from the Internet and still use it.

Most parts of Try In Browser are written in Rust, using [Seed](https://seed-rs.org/) web app framework.
This means the UI and interpreters will be fast and reliable in general.
The main challenge is that every interpreter should run the code in steps, in order to not freeze the entire browser.
This, along with the fact that the interpreters don't have the OS to back them up, implies that this approach is not applicable to
most practical, general-purpose languages. On the other hand, many esolangs don't need more than stdin/stdout, and most are quite simplistic and
easily run in steps, which makes TIB an ideal form of esolang showcase.

## Development instructions

Required tools: Rust, Cargo, cargo-make.

* You can install Rust and Cargo through [official instructions](https://www.rust-lang.org/tools/install).
* cargo-make: `cargo install cargo-make`

If you're using Linux and getting errors related to `openssl` and/or `pkg-config`, install the necessary packages via the package manager
(e.g. `sudo apt-get install libssl-dev pkg-config` for Ubuntu).

Provided `cargo make` commands:

* `cargo make build`: Build in development mode (quick build, large and slow Wasm binary)
* `cargo make build_release`: Build in release mode (longer build, optimized Wasm binary)
* `cargo make serve`: Run a server to see the application on the browser
* `cargo make verify`: Format and lint code