//! Alert rules over ldk-server events and polled state.
//!
//! Pure Rust with no I/O, so the same rules run in the sidecar and in the browser.
//! Must stay buildable for `wasm32-unknown-unknown`: CI checks it.
