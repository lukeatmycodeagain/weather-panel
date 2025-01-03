FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo fetch

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /usr/src/app/target/release/weather-micro .

EXPOSE 8080

CMD ["./weather-micro"]