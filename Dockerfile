FROM registry.naspersclassifieds.com/shared-services/core-services/rustbier/base-rust-image:latest

WORKDIR /usr/src/rustbier
COPY . .

RUN cargo test

RUN cargo install --path .

WORKDIR /

RUN rm -rf /usr/src

CMD rustbier