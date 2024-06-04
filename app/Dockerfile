
FROM rust:latest

WORKDIR /usr/app
COPY . .

RUN cargo install cargo-binstall

#==== DEBUG run
RUN cargo binstall cargo-watch --no-confirm
CMD ["cargo", "watch", "-x", "run", "--ignore", "static/*", "--ignore", "data/*"]

#==== PROD run
# RUN cargo build --release
# CMD ["./target/release/omotenashi-tokyo"]
