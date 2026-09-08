# SPDX-License-Identifier: Apache-2.0

{pkgs}: {
  packages = with pkgs; [
    bun
    just
    jq
    parallel
    quicktype
    foundry-bin
  ];
}
