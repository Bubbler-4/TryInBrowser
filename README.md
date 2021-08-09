# TryInBrowser

Online interpreter that works even if you go offline

## Why Try In Browser?

Most online interpreters require a server to run the submitted programs. And making different runs not interfere with each other (security concerns aside) is difficult.

Try In Browser attempts to solve this problem by moving all the interpreters into the browser. No need to isolate runs. No need to run a dedicated server;
a simple file server (such as GitHub Pages) is enough. An added benefit is that, once everything is loaded, you can disconnect from the Internet and still use it.

Most parts of Try In Browser are written in Rust, using [Seed](https://seed-rs.org/) web app framework.
This means the UI and interpreters will be fast and reliable in general.
Due to the fact that the interpreters don't have the OS to back them up, this approach is not generally applicable to
most practical, general-purpose languages. On the other hand, many esolangs don't need more than stdin/stdout, and most are quite simplistic and
easily run in steps, which makes TIB an ideal form of esolang showcase.

## What's new in TIB 2.0

TIB 2.0 is a major rewrite of the initial TIB.
The biggest difference is that the interpreter is run inside a Web Worker, so it can run at near-native speeds without blocking the UI.
This removes the "run-in-steps" requirement, which is pretty hard to achieve for many languages,
and opens up the possibility to include Rust-based runtimes such as RustPython.
Also, the interpreter can be stopped externally if it hangs (via `Worker.terminate()`), and a panic does not break the UI either.

## Distinctive features vs. other Try Online sites

* Disconnect from the server after page load and you can still use it.
* Output streaming. See partial output immediately as your code runs.
* No time limit. The only time limit is your patience.
  Other sites impose some time limit (between 5s and 60s) in order to control server load and escape from infinite loops. TIB doesn't need that.

## Convenience features

* Linkify: Permalink.
* Postify: Create a CGCC submission. Additionally, select some code by dragging to specify the main part.

---

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