# Build
FROM docker.io/library/rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime
FROM docker.io/library/debian:13-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      tini \
      ca-certificates \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/matrix-openrouter-helpdesk /app/matrix-openrouter-helpdesk

ENV METRICS_PORT=9090
EXPOSE 9090/tcp

ENTRYPOINT ["/usr/bin/tini", "--", "/app/matrix-openrouter-helpdesk"]
