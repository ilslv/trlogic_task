FROM rust:1 AS build

ENV USER=root

WORKDIR /code
RUN cargo new trlogic_task
WORKDIR /code/trlogic_task
COPY Cargo.toml ./Cargo.toml
RUN cargo build --release

COPY src ./src
RUN rm ./target/release/deps/trlogic_task*
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
COPY --from=build /code/trlogic_task/target/release/trlogic_task /
EXPOSE 8088
ENTRYPOINT [ "./trlogic_task" ]
