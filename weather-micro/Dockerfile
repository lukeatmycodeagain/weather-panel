FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY weather-micro ./weather-micro
COPY weather-utils ./weather-utils

WORKDIR /usr/src/app/weather-micro

RUN cargo fetch

RUN cargo build --release

FROM debian:bookworm-slim

ENV IS_CONTAINER="true"

COPY --from=builder /usr/src/app/weather-micro/target/release/weather-micro .

EXPOSE 8080

CMD ["./weather-micro"]