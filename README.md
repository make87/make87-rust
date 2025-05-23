![make87 Banner Logo](https://make87-files.nyc3.digitaloceanspaces.com/assets/branding/logo/make87_ME_1d_cv_cropped.svg)
# make87 SDK for Rust

## Overview

The make87 SDK for Rust provides tools and libraries to interact with the make87 platform. This SDK is designed to be compatible with Rust 2021 edition and supports optional features for different transports and encodings.

## Installation

To add the SDK to your project, include it in your `Cargo.toml`:

```toml
[dependencies]
make87 = "*"  # replace with latest version
```

### Optional Features

You can enable additional features for transport and encoding support:

- `zenoh` → Enables Zenoh transport (enables `interfaces::zenoh`)
- `protobuf` → Enables Protobuf encoding (enables `encodings::protobuf`)
- `yaml` → Enables YAML encoding (enables `encodings::yaml`)

Example:

```toml
[dependencies]
make87 = { version = "*", features = ["zenoh", "protobuf", "yaml"] }
```

## Usage

### Feature-gated Modules

- `interfaces::zenoh` is only available if the `zenoh` feature is enabled.
- `encodings::protobuf` is only available if the `protobuf` feature is enabled.
- `encodings::yaml` is only available if the `yaml` feature is enabled.


## Documentation
To build the documentation locally, use the following command:
```bash
cargo doc --open
```

## Contributing

We welcome contributions to the make87 SDK. Please follow these steps to contribute:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes.
4. Ensure all tests pass (`cargo test`).
5. Submit a pull request.

