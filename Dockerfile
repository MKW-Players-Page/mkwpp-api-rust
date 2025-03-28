# syntax=docker/dockerfile:1

FROM rust:1.85 AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .


## Image

FROM debian:bookworm

RUN apt-get update

COPY --from=builder /usr/local/cargo/bin/mkwpp-api-rust /usr/local/bin

CMD [ "mkwpp-api-rust" ]
