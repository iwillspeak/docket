FROM rust:latest as build
WORKDIR /usr/src/docket
COPY . .
RUN cargo install --path .

FROM debian:buster-slim as runtime
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/docket /usr/local/bin/docket
CMD ['/usr/local/bin/docket']