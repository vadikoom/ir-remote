FROM rust:1.68.1-alpine as builder
RUN apk update
RUN apk add libc-dev
RUN cargo init app
ADD backend/Cargo.toml /app
ADD backend/Cargo.lock /app
WORKDIR /app
RUN cargo build --release
RUN rm -rf src/*
COPY backend/ ./
RUN touch /app/src/main.rs
RUN cargo build --release

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/release/backend /app
COPY frontend/out /app/frontend
ENV STATIC_FILES_DIR=/app/frontend
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["/app/backend", "server"]
