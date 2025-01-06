FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY weather-server ./weather-server
COPY weather-utils ./weather-utils

WORKDIR /usr/src/app/weather-server

RUN cargo fetch

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

ENV ROCKET_PORT="8000"
ENV IS_CONTAINER="true"

WORKDIR /usr/src/app/weather-server

COPY --from=builder /usr/src/app/weather-server/target/release/weather-server .
COPY --from=builder /usr/src/app/weather-server/templates ./templates
COPY --from=builder /usr/src/app/weather-server/public ./public

EXPOSE 8000

CMD ["./weather-server"]