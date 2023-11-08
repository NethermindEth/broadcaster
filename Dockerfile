FROM --platform=$BUILDPLATFORM rust:1.73.0 AS build
ARG BUILDPLATFORM

WORKDIR /app

COPY . .
RUN cargo build
RUN cargo install --path .

ENTRYPOINT ["broadcaster"]
