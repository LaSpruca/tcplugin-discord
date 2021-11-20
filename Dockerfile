FROM rust
WORKDIR /app

EXPOSE 8080

COPY . .

RUN cargo build --release

CMD ["cargo", "run", "--release"]