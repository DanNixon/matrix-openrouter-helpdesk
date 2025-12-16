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

VOLUME ["/data"]
ENV MATRIX_SESSION_PATH=/data/matrix_session

ENV METRICS_ENDPOINT=0.0.0.0:9090
EXPOSE 9090/tcp

ENTRYPOINT ["/usr/bin/tini", "--", "/app/matrix-openrouter-helpdesk"]
