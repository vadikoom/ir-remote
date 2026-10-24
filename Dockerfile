FROM rust:1.99.1-alpine3.24 AS builder-rust
RUN apk add --no-cache git build-base pkgconfig openssl-dev
WORKDIR /app
COPY tgbot/Cargo.lock tgbot/Cargo.toml ./

# required to build the openssl crate, otherwise it crashes in runtime
ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release --locked && rm -r src
COPY tgbot/src ./src
COPY tgbot/config.yaml ./
RUN rm -f target/release/deps/tgbot*
RUN cargo build --release --locked


FROM alpine:3.24
WORKDIR /app
RUN apk add --no-cache ca-certificates openssl libgcc

COPY --from=builder-rust /app/target/release/tgbot ./backend

ENV IR_LISTEN_IP=0.0.0.0
ENV IR_LISTEN_PORT=12000
ENV RUST_BACKTRACE=1

CMD ["/app/backend"]
