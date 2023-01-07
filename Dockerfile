FROM rust:1.66

COPY ./ ./

EXPOSE 8080
RUN cargo build --release

RUN chmod +x ./entrypoint.sh

ENTRYPOINT [ "./entrypoint.sh" ]