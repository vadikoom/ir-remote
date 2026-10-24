# IR Remote Control

Control an air conditioner from Telegram using a Rust bot and an ESP8266 IR transmitter.
A personal hardware project, maintained for my own setup; adapting it to another appliance requires its IR timings.

The bot provides cooling, heating, dehumidification, off timers, and transmitter online/offline status.
The bundled controls use Ukrainian labels and Lessar/COOLIX IR signals.

```mermaid
flowchart LR
    classDef component fill:#f8f5f8,stroke:#b0a0b0,stroke-width:2px,color:#4a4a4a
    classDef queue fill:#fff0f0,stroke:#d4a0a0,stroke-width:2px,color:#4a4a4a
    classDef external fill:#f5f5f3,stroke:#999988,stroke-width:2px,color:#4a4a4a
    classDef store fill:#f0f5f0,stroke:#88a888,stroke-width:2px,color:#3a4a3a
    classDef dim fill:#f5f5f3,stroke:#c0b8b0,color:#888880,stroke-width:1.5px,stroke-dasharray:5 5
    TG[Telegram]:::external <-->|Telegram API| Bot[Rust bot]:::component
    Bot <-->|UDP / JSON| ESP[ESP8266]:::component
    ESP -->|38 kHz IR| AC[Air conditioner]:::external
```

## What's here

| Directory | Purpose |
| --- | --- |
| [`tgbot/`](tgbot/) | Telegram bot, command scripts, timers, and UDP server |
| [`firmware/controller/`](firmware/controller/) | ESP8266 transmitter firmware; IR output on GPIO2 |
| [`firmware/tester/`](firmware/tester/) | Separate Arduino Mega capture device; IR receiver on pin 11 |
| [`firmware/lib/`](firmware/lib/) | Capture decoder support |
| [`infra/`](infra/) | Optional Docker Compose deployment over SSH |

## Running it

Install Docker and [Task](https://taskfile.dev), then create your local configuration:

```sh
cp .env.example .env
# Edit .env with your Telegram token, user ID, Wi-Fi, and server address.
task build
task image:run
```

The container listens on UDP port 12000, published as `IR_LISTEN_PORT` (4944 in the example).
The firmware sends to port 4944 on `FIRMWARE_REMOTE_HOST`; that address must be reachable from the device.
Existing `.vadikk.com.env` configurations remain supported; `.env` takes precedence when both files exist.

For the transmitter, the default board is NodeMCU ESP8266. Connect your IR transmitter circuit to GPIO2,
set the upload port in [`firmware/taskfile.yaml`](firmware/taskfile.yaml), and run:

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

Edit [`tgbot/config.yaml`](tgbot/config.yaml) to change buttons, raw IR timings, responses, or command sequences.
It is embedded in the binary, so rebuild after editing. Status reports transmitter connectivity, not appliance state.
The capture device is an independent tool for inspecting remote signals, with its own build/upload tasks.
Capture builds additionally need `IRremote` installed in the build image. Its serial output uses 9600 baud;
the controller uses 115200.

Run `cargo test --locked` inside `tgbot/` for IR signal, authorization, timer, Telegram API, and UDP checks.
Run `cargo clippy --all-targets --locked -- -D warnings` for Rust lint checks, or `task firmware:test`
for the capture decoder diagnostic. Capture diagnostics print decoded data; they are not assertion-based tests.
