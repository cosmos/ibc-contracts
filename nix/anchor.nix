# SPDX-License-Identifier: Apache-2.0

{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  openssl,
  perl,
  hidapi,
  udev,
}:
rustPlatform.buildRustPackage rec {
  pname = "anchor";
  version = "0.32.1";

  src = fetchFromGitHub {
    owner = "solana-foundation";
    repo = "anchor";
    tag = "v${version}";
    hash = "sha256-oyCe8STDciRtdhOWgJrT+k50HhUWL2LSG8m4Ewnu2dc=";
    fetchSubmodules = true;
  };

  cargoHash = "sha256-XrVvhJ1lFLBA+DwWgTV34jufrcjszpbCgXpF+TUoEvo=";

  nativeBuildInputs = [perl pkg-config];

  buildInputs = [openssl] ++ lib.optionals stdenv.isLinux [hidapi udev];

  checkFlags = [
    "--skip=tests::test_check_and_get_full_commit_when_full_commit"
    "--skip=tests::test_check_and_get_full_commit_when_partial_commit"
    "--skip=tests::test_get_anchor_version_from_commit"
  ];

  meta = with lib; {
    description = "Solana Sealevel Framework";
    homepage = "https://github.com/solana-foundation/anchor";
    changelog = "https://github.com/solana-foundation/anchor/blob/${src.rev}/CHANGELOG.md";
    license = licenses.asl20;
    mainProgram = "anchor";
  };
}
