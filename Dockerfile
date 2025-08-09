FROM rust:latest
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build
EXPOSE 3000
CMD ["./target/debug/server"]