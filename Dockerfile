FROM rust:1.61

COPY ./ ./

RUN cargo build --release
RUN rm ./src/*.rs

CMD ["./target/release/rusty-cards"]