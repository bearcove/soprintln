# soprintln

This is only useful when implementing the `xgraph` dynamic linking model,
see <https://github.com/bearcove/rubicon>

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
