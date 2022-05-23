FROM rust:1.61 as builder

WORKDIR /workspace

RUN apt update
RUN apt install protobuf-compiler -y
RUN protoc --version

COPY . .
RUN cargo install --path .

FROM gcr.io/distroless/cc:nonroot
WORKDIR /app
COPY --from=builder /workspace/target/release/google-cloud-rust-example-pubsub .

CMD ["/app/google-cloud-rust-example-pubsub"]