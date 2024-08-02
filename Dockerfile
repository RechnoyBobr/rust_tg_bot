FROM rust
WORKDIR /usr/src/rust_bot
COPY . .
RUN cargo install --path .
CMD ["rust_tg_bot"]
