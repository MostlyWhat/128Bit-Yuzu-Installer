## 128Bit-Yuzu Installer

## About

This is installer is programmed for the 128BitBay Server. It is designed to update your yuzu installation.

## Changelog

This is the changelog for the 128Bit-Yuzu Installer, it contains all the changes made to the installer since the last release.

- Version 0.3: Implementing quality of life updates
- Version 0.2: Faster Backend and Download Systems
- Version 0.1: Initial Release

## Building

For more detailed instructions, look at the usage documentation above.

There are are few system dependencies for windows:

- `cargo` should be available on your PATH. [Rustup](https://rustup.rs/) is the
  recommended way to achieve this. Stable or Nightly Rust works fine.
- Have node.js and Yarn available on your PATH (for building UI components, not needed at runtime).
- For Windows (MSVC), you need Visual Studio installed.
- For Windows (Mingw), you need `gcc`/`g++` available on the PATH.

In order to build yourself an installer, you need to:

```bash
cargo build --release
```

## Contributing

PRs are very welcome. Code should be run through [Rustfmt](https://github.com/rust-lang-nursery/rustfmt)
before submission.

## License

128Bit Yuzu Installer is licensed under the Apache 2.0 License, which can be found in [LICENSE](LICENSE).
