```bash
# Install Go.
sudo snap install go --classic

# Install protobuf compiler.
PROTOC_ZIP=protoc-3.11.1-linux-x86_64.zip
curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v3.11.1/$PROTOC_ZIP
sudo unzip -o $PROTOC_ZIP -d /usr/local bin/protoc
sudo unzip -o $PROTOC_ZIP -d /usr/local 'include/*'
rm -f $PROTOC_ZIP
sudo chmod +x /usr/local/bin/protoc
sudo chmod +r -R /usr/local/include/google

# Install grpc-gateway packages.
# <https://github.com/grpc-ecosystem/grpc-gateway>
go get -u google.golang.org/grpc
go get -u google.golang.org/genproto/googleapis
go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway
go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger
go get -u github.com/golang/protobuf/protoc-gen-go

# Copy protocol files to `/usr/local/include`.
sudo cp -r $HOME/go/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/api /usr/local/include/google
sudo cp -r $HOME/go/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/rpc /usr/local/include/google

# Generate grpc-gateway reverse proxy and OpenAPI file.
sso_grpc/proto
GOBIN="$HOME/go/bin"
PATH="$PATH:$GOBIN"
protoc -I/usr/local/include -I. \
  -I$HOME/go/src \
  --go_out=plugins=grpc:. \
  sso.proto
protoc -I/usr/local/include -I. \
  -I$HOME/go/src \
  --grpc-gateway_out=logtostderr=true:. \
  sso.proto
protoc -I/usr/local/include -I. \
  -I$HOME/go/src \
  --swagger_out=logtostderr=true:. \
  sso.proto

# Build OpenAPI gateway.
cd sso_openapi
go build -o sso-openapi-server

# Build and run gRPC server.
cd sso_grpc
cargo build
cargo run
```
