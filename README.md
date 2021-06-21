<img src="banner.png" width="400px" />
    
[Usage Documentation](https://liftinstall.jselby.net/) 
    - [Quick Start](https://liftinstall.jselby.net/quick-start) 
    - [License](LICENSE)

[![Build Status](https://travis-ci.org/j-selby/liftinstall.svg?branch=master)](https://travis-ci.org/j-selby/liftinstall)


An installer for your application. Designed to be customisable to the core, hookable from external
 applications, and have a decent UI.

This is designed to be a more modern interpretation of Qt's Installer Framework, which is hard to develop on,
 poorly documented, has a hardcoded package listing format, and isn't supported all that well, with rare updates 
 and a large list of bugs.

Building
--------

For more detailed instructions, look at the usage documentation above.

There are are few system dependencies depending on your platform:
- For all platforms, `cargo` should be available on your PATH. [Rustup](https://rustup.rs/) is the 
  recommended way to achieve this. Stable or Nightly Rust works fine.
- Have node.js and Yarn available on your PATH (for building UI components, not needed at runtime).
- For Windows (MSVC), you need Visual Studio installed.
- For Windows (Mingw), you need `gcc`/`g++` available on the PATH.
- For Mac, you need Xcode installed, and Clang/etc available on the PATH.
- For Linux, you need `gcc`/`g++`, `webkit2gtk`, and `libssl`. For Ubuntu 18.04 this would look like:

```bash
apt install -y build-essential libwebkit2gtk-4.0-dev libssl-dev
```

In order to build yourself an installer, as a bare minimum, you need to:

- Add your favicon to `ui/public/favicon.ico`
- Add your logo to `ui/src/assets/logo.png`
- Modify the bootstrap configuration file as needed (`config.PLATFORM.toml`).
- Have the main configuration file somewhere useful, reachable over HTTP.
- Run:

```bash
cargo build --release
```

Contributing
------------

PRs are very welcome. Code should be run through [Rustfmt](https://github.com/rust-lang-nursery/rustfmt) 
 before submission.

License
-------

LiftInstall is licensed under the Apache 2.0 License, which can be found in [LICENSE](LICENSE).
