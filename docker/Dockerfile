FROM rust:1.95-trixie AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian13:nonroot

WORKDIR /app

COPY --from=builder /app/target/release/waline-server /usr/local/bin/waline-server

EXPOSE 8360

ENTRYPOINT ["waline-server"]