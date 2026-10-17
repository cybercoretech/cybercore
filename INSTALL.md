# Install the Cybercore theme checker

Cybercore 0.8 publishes `cybercore-theme`, a command-line tool for checking
theme quality, compatibility, and portable theme packs. Install Rust and Cargo
first, then run:

```sh
curl -fsSL https://cybercoretech.github.io/cybercore/install.sh | sh
```

The installer requires Cargo and installs only the `cybercore-theme` binary. It
does not install the Rust library or the standalone Cybercore Theme Studio.
For Rust applications, add `cybercore = "0.8"` to `Cargo.toml`. The equivalent
direct CLI installation command is:

```sh
cargo install --locked cybercore --bin cybercore-theme
```

The script source is [`install.sh`](install.sh).
