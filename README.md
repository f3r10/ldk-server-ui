# ldk-server-ui

An operator console for [ldk-server](https://github.com/lightningdevkit/ldk-server): a Rust/WASM
front end that talks to ldk-server through a sidecar (and, later, directly over gRPC-Web), with
alerts and notifications.

Early development.

## Layout

| Crate | Role | Targets |
|---|---|---|
| `crates/api` | Wire types shared by sidecar and front end | native, wasm32 |
| `crates/alerts` | Alert rules, pure Rust, no I/O | native, wasm32 |
| `crates/notify` | Out-of-band delivery (Telegram first) | native |
| `crates/sidecar` | Serves the UI, proxies to ldk-server via `ldk-server-client` | native |
| `crates/web` | Browser front end | wasm32 |

## Toolchain

`rust-toolchain.toml` pins Rust 1.95.0 with `rustfmt`, `clippy` and the
`wasm32-unknown-unknown` target. Only rustup's `cargo` honours it; rustup installs
the pinned toolchain on first use.

On macOS with both the Homebrew `rust` and `rustup` formulae installed, Homebrew's
`rust` wins on `PATH` and has no wasm32 standard library. Put rustup's proxies first
(fish: `fish_add_path -m /opt/homebrew/opt/rustup/bin`), or `brew uninstall rust`.
Check with:

```bash
which cargo   # should be /opt/homebrew/opt/rustup/bin/cargo (or ~/.cargo/bin/cargo)
```

`rustfmt.toml` is ldk-server's. Stable `rustfmt` ignores its nightly-only options
and prints warnings; ldk-server's CI also uses stable.

Checks that CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --target wasm32-unknown-unknown -p api -p alerts -p web
```

## Development environment

Development uses [`ldk-lab`](../ldk-lab), a separate regtest setup next to this repo:
bitcoind plus two ldk-server nodes in Docker Compose.

| | gRPC from the host | TLS certificate | Admin macaroon |
|---|---|---|---|
| node a | `127.0.0.1:3536` | `../ldk-lab/data/a/tls.crt` | `../ldk-lab/data/a/regtest/macaroons/admin.macaroon` |
| node b | `127.0.0.1:3537` | `../ldk-lab/data/b/tls.crt` | `../ldk-lab/data/b/regtest/macaroons/admin.macaroon` |

```bash
cd ../ldk-lab
./scripts/compose up -d
./scripts/fund
./scripts/open-channel 100000sat
./scripts/mine 6 a
```

**API version.** The ldk-server crates are pinned in `Cargo.toml` to
`c17dc11` (upstream `main`, 2026-10-15). The lab node must speak the same protobuf
API. Any lab image built from a checkout whose `ldk-server-grpc`, `ldk-server-client`,
`ldk-server-macaroons` and `ldk-server/src` match that commit is compatible. Check
before bumping:

```bash
git -C <checkout> diff --stat c17dc11 HEAD -- ldk-server-grpc ldk-server-client ldk-server-macaroons ldk-server/src
```

Note: upstream's `Dockerfile` at that commit does not copy `ldk-server-macaroons`, so
the lab image only builds from a checkout with that fixed locally
([#320](https://github.com/lightningdevkit/ldk-server/issues/320)).

## Licence

MIT OR Apache-2.0, at your option. See [LICENSE.md](LICENSE.md).
