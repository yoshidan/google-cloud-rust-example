FROM rust:1.58

WORKDIR /usr/src/google-cloud-rust-example/pubsub/rust
COPY . .

RUN cargo install --path .

CMD ["google-cloud-rust-example-spanner"]