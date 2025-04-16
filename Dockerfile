FROM rust:1.86-alpine3.21 AS builder
LABEL authors="tapnisu"

WORKDIR /usr/src/lyceumnstubot

RUN apk update \
    && apk upgrade --available \
    && apk add --no-cache alpine-sdk libressl-dev

COPY . .
RUN cargo build --release

FROM alpine:3.21 AS runner

RUN apk update \
    && apk upgrade --available \
    && apk add --no-cache ca-certificates \
    && update-ca-certificates

COPY --from=builder /usr/src/lyceumnstubot/target/release/lyceumnstubot /usr/local/bin/lyceumnstubot

CMD ["lyceumnstubot"]
