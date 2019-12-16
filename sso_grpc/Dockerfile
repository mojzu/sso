FROM sso_build:latest as build

# Copy files and build binary.
ADD . /sso/sso_grpc
RUN cargo make release;

FROM debian:10.2
ENV DEBIAN_FRONTEND="noninteractive"

# Copy and start server binary.
COPY --from=build /sso/target/release/sso-grpc-server .
CMD ["./sso-grpc-server"]
