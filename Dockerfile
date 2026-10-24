FROM rust:1.70-alpine as builder-rust
RUN apk update && apk add --no-cache git build-base pkgconfig openssl-dev
WORKDIR /app
COPY tgbot/Cargo.lock tgbot/Cargo.toml ./

# required to build the openssl crate, otherwise it crashes in runtime
ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release && rm -r src
COPY tgbot/src ./src
COPY tgbot/config.yaml ./
RUN rm -f target/release/deps/tgbot*
RUN cargo build --release


FROM alpine:latest
WORKDIR /app
RUN apk update && apk add openssl libgcc

COPY --from=builder-rust /app/target/release/tgbot ./backend

ENV IR_LISTEN_IP=0.0.0.0
ENV IR_LISTEN_PORT=12000
ENV RUST_BACKTRACE=1

CMD ["/app/backend"]
