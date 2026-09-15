# SPDX-License-Identifier: Apache-2.0

{pkgs}: {
  packages = with pkgs; [
    go
    gopls
    gofumpt
    golangci-lint
    gocyclo
    # Proto tooling
    protobuf
    buf
    protoc-gen-go
    protoc-gen-go-grpc
  ];
}
