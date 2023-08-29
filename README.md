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

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.
