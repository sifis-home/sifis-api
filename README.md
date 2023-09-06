# [![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/sifis-home/sifis-api/workflows/sifis-api/badge.svg)](https://github.com/sifis-home/sifis-api/actions)
[![Crates.io](https://img.shields.io/crates/v/sifis.svg)](https://crates.io/crates/sifis)
[![dependency status](https://deps.rs/repo/github/sifis-home/sifis-api/status.svg)](https://deps.rs/repo/github/sifis-home/sifis-api)
[![Documentation](https://docs.rs/sifis/badge.svg)](https://docs.rs/sifis/)

[SIFIS-Home](https://sifis-home.eu) developer API.

**NOTE**: This repo history will be rewritten to be fully descriptive.

## Supported Devices

- [x] Lamp
- [x] Sink
- [x] Door
- [x] Fridge

## Usage

The library crate by default opens a unix socket on `/var/run/sifis.sock` or to the path set in the env var `SIFIS_SERVER`.

## Testing

The crate provides two developer tools:
- `sifis-runtime-mock`: a `runtime` example implementation that simulates devices, useful to implement mock testing of client applications.
- `sifis-client`: an interactive client to help developing independent runtimes and explore the overall API.

``` sh
# Change the default unix socket path
export SIFIS_SERVER=/tmp/sifis.sock

# Start the runtime with the default configuration
cargo run --bin sifis-runtime-mock &

# Start the interactive client
cargo run --bin sifis-client
```

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.
