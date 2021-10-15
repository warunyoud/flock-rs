FROM rust:1.49 as builder
WORKDIR /usr/src/flight
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/flight /usr/local/bin/flight
CMD ["flight"]