# SPDX-License-Identifier: Apache-2.0

{ pkgs }:

pkgs.buildGoModule rec {
  pname = "protoc-gen-gocosmos";
  version = "1.5.0";

  src = pkgs.fetchFromGitHub {
    owner = "cosmos";
    repo = "gogoproto";
    rev = "v${version}";
    sha256 = "sha256-AvEoxkTNNTBGHffokrsQ5z/A+WXjNqmefy25wtogXFw=";
  };

  vendorHash = "sha256-hD2ssRJk1uE/LnUNJkuNmMu3Ogdi5iEHN2qJZszNEh8=";

  subPackages = [ "protoc-gen-gocosmos" ];

  doCheck = false;

  meta = with pkgs.lib; {
    description = "Cosmos Protocol Buffers for Go with Gadgets";
    homepage = "https://github.com/cosmos/gogoproto";
    license = licenses.bsd3;
    maintainers = with maintainers; [ ];
  };
}
