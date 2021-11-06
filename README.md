# `ftdi`

This is a library wrapping `libftdi1-sys` and providing a more idiomatic interface for working with FTDI devices in Rust.

# MSRV

At the moment the MSRV of this crate is Rust 1.40.0.

# Changelog

## 0.1.3

MSRV increased from 1.34.0 to 1.40.0.

Additions:

- extended MPSSE support via `ftdi-mpsse`
- event char and error char configuration
- `libftdi` context getter

## 0.1.2

Additions:

- bitmode configuration
- line properties configuration
- flow control configuration
- baud rate configuration
- a number of useful derives on Interface

## 0.1.1

Additions:

- a `vendored` feature for `libftdi1-sys`.

## 0.1.0

The initial release with something like a stable API.
