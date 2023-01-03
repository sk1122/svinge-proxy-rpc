FROM rust:1.66

COPY ./ ./

EXPOSE 8080
RUN cargo build --release

CMD [ "./target/release/svinge" ]