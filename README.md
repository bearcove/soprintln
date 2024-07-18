[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/soprintln.svg)](https://crates.io/crates/soprintln)
[![docs.rs](https://docs.rs/soprintln/badge.svg)](https://docs.rs/soprintln)

# soprintln

(Note: This is only useful when implementing the `xgraph` dynamic linking model,
see <https://github.com/bearcove/rubicon>)

![](https://github.com/user-attachments/assets/3bc0e0e1-cade-4b27-88b5-5d73029a0e74)

This crate provides the `soprintln!` macro, a debug variant of `println!` that:

  * Is disabled if the `print` feature cargo feature is not enabled
  * Is disabled if the `SOPRINTLN` environment variable isn't set to one
  * Prefixes the message with:
   * a truncated millisecond timestamp
   * a beacon of the shared object ID
   * a beacon of the thread name + ID

Beacons are 64-bit integers (can be initialized from pointers) whose color
depend on their value. It makes it easier to spot the same value being re-used
a bunch.
