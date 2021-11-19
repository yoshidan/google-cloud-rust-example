FROM rust:1.56

WORKDIR /usr/src/google-cloud-rust-example
COPY . .

ARG GITHUB_TOKEN
RUN git config --global url."https://${GITHUB_TOKEN}:x-oauth-basic@github.com/".insteadOf "https://github.com/"
RUN cargo install --path .

CMD ["google-cloud-rust-example"]