FROM rust:1.73.0

WORKDIR /app

COPY . .
RUN cargo build
RUN cargo install --path .

ENTRYPOINT ["broadcaster"]
