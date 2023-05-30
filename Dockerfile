FROM rust:1.69 as builder
WORKDIR /usr/src/tot_spec
COPY . .
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install --path codegen

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/tot_spec /usr/local/bin/tot_spec
CMD ["tot_spec"]
