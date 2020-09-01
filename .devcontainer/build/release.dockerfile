FROM sso/build-cache:latest

# Rust first due to compilation time
COPY ./sso /build/sso
RUN cargo build --release

# Node
COPY ./sso_test /build/sso_test
RUN npm run tsc
