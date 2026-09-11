FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-api-drive-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-api-drive-standalone-gateway /usr/local/bin/sdkwork-api-drive-standalone-gateway
ENV SDKWORK_DRIVE_STANDALONE_GATEWAY_BIND=0.0.0.0:3900
EXPOSE 3900
CMD ["sdkwork-api-drive-standalone-gateway"]
