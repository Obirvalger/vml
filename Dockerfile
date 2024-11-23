FROM rust:1.75

WORKDIR /usr/src/vml

COPY src /usr/src/vml/src
COPY vendor /usr/src/vml/vendor
COPY files /usr/src/vml/files
COPY Cargo.toml /usr/src/vml/Cargo.toml
COPY Cargo.lock /usr/src/vml/Cargo.lock
COPY .cargo /usr/src/vml/.cargo

RUN cargo build --release
