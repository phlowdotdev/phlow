FROM rust:latest

RUN cargo install cross

WORKDIR /app

COPY . .

RUN cross build --target aarch64-unknown-linux-gnu --release -p phlow-runtime
RUN cross build --target x86_64-unknown-linux-gnu --release -p phlow-runtime
