# Setup and development

Install Docker and [Task](https://taskfile.dev), then create your local configuration from the repository root:

```sh
cp .env.example .env
# Fill in the bot token, allowed user IDs, Wi-Fi, server address, and shared key.
# Generate the shared key with: openssl rand -hex 32
task build
task image:run
```

The container listens on UDP port 12000, published as `IR_LISTEN_PORT` (4944 in the example).
The firmware sends to port 4944 on `FIRMWARE_REMOTE_HOST`; that address must be reachable from the device.
Existing `.vadikk.com.env` configurations remain supported; `.env` takes precedence when both files exist.

For the transmitter, the default target is an ESP8266 with 1MB flash and NodeMCU-style automatic reset.
Connect your IR transmitter circuit to GPIO2,
set the upload port in [`firmware/taskfile.yaml`](../firmware/taskfile.yaml), and run:

```sh
task firmware:setup
task firmware:build:controller
task firmware:upload:controller
```

Uploading requires `arduino-cli` on the host with the ESP8266 core installed; compilation runs in Docker.
For local bot development, install Rust with rustup and run `task tgbot:dev` (binds to `127.0.0.1:4944`).
The toolchain is pinned to Rust 1.99.1 in `rust-toolchain.toml`; the bot uses edition 2021.
For optional SSH deployment, fill in the deployment variables in `.env` and use `task deploy` after building.

## Adapting and checking

Edit [`tgbot/config.yaml`](../tgbot/config.yaml) to change buttons, raw IR timings, responses, or command sequences.
It is embedded in the binary, so rebuild after editing. Status reports transmitter connectivity, not appliance state.
The capture device is an independent tool for inspecting remote signals, with its own build/upload tasks.
Capture builds additionally need `IRremote` installed in the build image. Its serial output uses 9600 baud;
the controller uses 115200.

Run `cargo test --locked` inside `tgbot/` for IR signal, authorization, timer, Telegram API, and UDP checks.
Run `cargo clippy --all-targets --locked -- -D warnings` for Rust lint checks, or `task firmware:test`
for firmware security tests and the capture decoder diagnostic. Security tests use host stubs with address and
undefined-behavior sanitizers. Capture diagnostics print decoded data; they are not assertion-based tests.

See [protocol and security](protocol.md) for shared-key setup and delivery guarantees.
