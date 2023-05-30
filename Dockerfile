FROM rust:1.69 as builder
WORKDIR /usr/src/tot_spec
COPY . .
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install --path codegen

FROM debian:bullseye-slim
RUN apt-get update \
  && apt-get install -y make  \
  && apt-get clean \
  && apt-get autoremove -y \
  && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY --from=builder /usr/local/cargo/bin/tot_spec /usr/local/bin/tot_spec

CMD ["tot_spec"]
