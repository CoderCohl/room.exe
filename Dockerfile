FROM rust:1.79 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/room.exe /usr/local/bin/room.exe
RUN useradd -r -s /usr/sbin/nologin room || true
RUN mkdir -p /var/lib/room.exe/rooms /var/lib/room.exe/backups && chown -R room:room /var/lib/room.exe
USER room
WORKDIR /var/lib/room.exe
ENTRYPOINT ["/usr/local/bin/room.exe"]
CMD ["--config","/var/lib/room.exe/config.json","daemon"]

