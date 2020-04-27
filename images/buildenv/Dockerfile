FROM rust:1.31
MAINTAINER snlan
ENV GOCACHE=off

RUN apt-get update && apt-get install -y --no-install-recommends unzip

# protoc
RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        i386) url='https://github.com/protocolbuffers/protobuf/releases/download/v3.6.1/protoc-3.6.1-linux-x86_32.zip' ;; \
        amd64) url='https://github.com/protocolbuffers/protobuf/releases/download/v3.6.1/protoc-3.6.1-linux-x86_64.zip' ;; \
    esac; \
    wget -q -O protoc.zip "$url"; \
    unzip -d /usr/local protoc.zip; \
    rm protoc.zip

# protoc-gen-go
RUN go get -u -v -ldflags '-w -s' github.com/golang/protobuf/protoc-gen-go
