FROM rust:latest AS builder

ENV ROCKET_PORT=8000
ENV IS_CONTAINER=true

WORKDIR /usr/src/app

COPY . .

RUN cargo fetch

RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/app/target/release/weather-server .

EXPOSE 8000

CMD ["./weather-server"]