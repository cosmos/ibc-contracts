# SPDX-License-Identifier: Apache-2.0

{
  description = "Development environment for Solidity IBC Eureka";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    solc = {
      url = "github:hellwolf/solc.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    foundry.url = "github:shazow/foundry.nix/stable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    natlint = {
      url = "github:srdtrk/natlint";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    sp1 = {
      url = "github:vaporif/sp1-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachSystem
    ["x86_64-linux" "aarch64-linux" "aarch64-darwin"]
    (
      system: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (import inputs.rust-overlay)
            inputs.foundry.overlay
            inputs.solc.overlay
            inputs.sp1.overlays.default
          ];
        };

        rust = import ./nix/rust.nix {inherit pkgs;};
        go = import ./nix/go.nix {inherit pkgs;};
        common = import ./nix/common.nix {inherit pkgs;};
        solidity = import ./ibc-solidity/nix/solidity.nix {inherit pkgs inputs system;};
        node-modules = import ./ibc-solidity/nix/node-modules.nix {inherit pkgs;};
        anchor = pkgs.callPackage ./nix/anchor.nix {};
        solana-agave = pkgs.callPackage ./nix/agave.nix {
          inherit (pkgs) rust-bin;
          inherit anchor;
        };
        anchor-go = pkgs.callPackage ./nix/anchor-go.nix {};

        # Always replace `ibc-solidity/node_modules` with the Nix-managed one so that the shell never
        # picks up a stale local install, whether it is a symlink or a real directory.
        #
        # The hook only knows the caller's working directory, not where the flake was loaded from, so
        # before deleting anything it checks that the directory is a checkout of this repository whose
        # lockfile matches the one `node-modules` was built from. Running `nix develop <this repo>`
        # from an unrelated checkout therefore leaves that checkout's `node_modules` untouched.
        linkNodeModules = ''
          if [ -d "${node-modules}/node_modules" ]; then
            repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            solidity_dir="$repo_root/ibc-solidity"
            if cmp -s "${node-modules.src}/package.json" "$solidity_dir/package.json" \
              && cmp -s "${node-modules.src}/bun.lock" "$solidity_dir/bun.lock"; then
              rm -rf "$solidity_dir/node_modules"
              ln -sfn "${node-modules}/node_modules" "$solidity_dir/node_modules"
            else
              echo "warning: $solidity_dir does not match this flake's ibc-solidity; leaving node_modules alone" >&2
            fi
          fi
        '';
      in {
        devShells = {
          default = pkgs.mkShell {
            buildInputs =
              rust.packages
              ++ go.packages
              ++ common.packages
              ++ solidity.packages
              ++ [
                node-modules
              ]
              ++ (with pkgs.sp1."v6.1.0"; [
                cargo-prove
                sp1-rust-toolchain
              ]);
            inherit (rust) NIX_LD_LIBRARY_PATH;
            inherit (rust.env) RUST_SRC_PATH;
            shellHook =
              rust.shellHook
              + ''
                ${linkNodeModules}
              '';
          };

          # Everything needed to work on the Solana part of this project
          solana = pkgs.mkShell {
            buildInputs =
              rust.packages
              ++ go.packages
              ++ common.packages
              ++ [solana-agave anchor-go node-modules];
            inherit (rust.env) RUST_SRC_PATH;
            shellHook =
              rust.shellHook
              + ''
                ${linkNodeModules}

                export PATH="${solana-agave}/bin:$PATH"
                echo "Solana shell: solana, anchor-nix (build|test|unit-test|keys|deploy)"
              '';
          };
        };
      }
    );
}
