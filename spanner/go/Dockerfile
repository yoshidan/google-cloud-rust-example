ARG GOLANG_TAG=1.17-alpine

FROM golang:${GOLANG_TAG} as builder

RUN apk add -U --no-cache git gcc libc-dev
WORKDIR /go/src/github.com/yoshidan/google-cloud-rust-example/spanner/go

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN go build -o build/main cmd/main.go

FROM alpine:3.13

RUN mkdir /app && \
  apk add -U --no-cache ca-certificates tzdata
COPY --from=builder /go/src/github.com/yoshidan/google-cloud-rust-example/spanner/go /app/
CMD ["/app/build/main"]