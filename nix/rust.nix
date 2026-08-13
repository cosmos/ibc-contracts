# SPDX-License-Identifier: Apache-2.0

{pkgs}: let
  toolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = ["rust-src" "rust-analyzer" "clippy" "rustfmt"];
  };
in {
  packages = with pkgs;
    [
      toolchain
      pkg-config
      openssl
      openssl.dev
      sccache
      cargo-nextest
    ]
    ++ lib.optionals stdenv.isDarwin [
      apple-sdk_15
    ];

  env = {
    RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
  };

  NIX_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.stdenv.cc.cc];

  shellHook = ''
    # WORKAROUND: Fix Darwin SDK conflicts
    # nixpkgs unstable has mismatched Apple SDK versions causing linker errors.
    if [[ "$OSTYPE" == "darwin"* ]]; then
      unset DEVELOPER_DIR_FOR_TARGET
      unset NIX_APPLE_SDK_VERSION_FOR_TARGET
      unset SDKROOT_FOR_TARGET
    fi
  '';

  inherit toolchain;
}
