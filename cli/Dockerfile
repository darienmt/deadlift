FROM rust:1.80 as build

WORKDIR /build

COPY ../Cargo.* ./
COPY ../engine ../engine
COPY . .

RUN cargo build -p cli --release

FROM ubuntu:latest as host

COPY --from=build /build/target/release/deadlift /usr/local/bin/deadlift

CMD ["deadlift"]
