FROM rust:1.56

WORKDIR /usr/src/google-cloud-rust-example/spanner/rust
COPY . .

RUN cargo install --path .

CMD ["google-cloud-rust-example-spanner"]