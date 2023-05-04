FROM rust:latest AS builder

RUN USER=root cargo new --bin git_history_explorer
WORKDIR /git_history_explorer

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release
RUN rm src/*.rs

COPY src ./src

RUN cargo build --release


FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /git_history_explorer/target/release/git_history_explorer /usr/local/bin/

ENTRYPOINT ["git_history_explorer"]