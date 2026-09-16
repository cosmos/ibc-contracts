# abigen pinned to the go-ethereum release CI uses for `just solidity::generate-abi`.
#
# nixpkgs' `go-ethereum` package lags upstream (1.17.3 while CI pins 1.17.5), and
# abigen releases change the generated boilerplate, so regenerating bindings with the
# nixpkgs binary dirties every file under packages/go-abigen and fails the abigen CI
# check. go-ethereum ships no flake, and nix-community/ethereum.nix restricts its geth
# package (which provides abigen) to Linux, so we build cmd/abigen ourselves.
#
# Keep `version`/`rev` in sync with the abigen pin in .github/workflows/abigen.yaml
# and the version check in ibc-solidity/solidity.just.
{pkgs}:
pkgs.buildGoModule rec {
  pname = "abigen";
  version = "1.17.5";

  src = pkgs.fetchFromGitHub {
    owner = "ethereum";
    repo = "go-ethereum";
    rev = "v${version}"; # 9621c6ad10934a01b5514886fb6fbd87640b6c05
    hash = "sha256-KuXriZP3qMpChRF5hcQP2ZlmqUF5k+WcutDr3/oAI/0=";
  };

  proxyVendor = true;
  vendorHash = "sha256-kfVO/yeCDzYfKY4OWW6slT8Y2xrfO8BruKIcCZWW2P0=";

  subPackages = ["cmd/abigen"];
  # Following upstream build/ci.go.
  tags = ["urfave_cli_no_docs"];
  ldflags = ["-s" "-w"];
  doCheck = false;

  meta = with pkgs.lib; {
    description = "Go binding generator for Ethereum contracts (go-ethereum cmd/abigen)";
    homepage = "https://geth.ethereum.org/docs/tools/abigen";
    license = licenses.gpl3Plus;
    mainProgram = "abigen";
  };
}
