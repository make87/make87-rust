![make87 Banner Logo](docs/src/assets/images/make87_ME_1d_cv_cropped.svg)
# make87 SDK for Rust

## Overview

The make87 SDK for Rust provides tools and libraries to interact with the make87 platform. This SDK is designed to be compatible with Rust 2021 edition.

## Installation

To add the SDK to your project, include it in your `Cargo.toml`:

```toml
[dependencies]
make87 = "*"  # replace with latest version
```

### Dependencies

The SDK has the following dependencies:

- `once_cell = "1.20.2"`
- `prost = "0.13"`
- `serde = { version = "1.0.210", features = ["derive"] }`
- `serde_json = "1.0.128"`
- `thiserror = "1.0.63"`
- `tokio = { version = "1.16.1", features = ["time"] }`
- `zenoh = { version = "1.2.1", features = ["unstable"] }`

## Documentation
To build the documentation locally, use the following command:
```bash
cargo doc --open
```

Then, build the documentation using MkDocs:

```bash
mkdocs build
```

## Contributing

We welcome contributions to the make87 SDK. Please follow these steps to contribute:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes.
4. Ensure all tests pass.
5. Submit a pull request.
