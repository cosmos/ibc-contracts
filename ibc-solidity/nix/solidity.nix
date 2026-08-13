# SPDX-License-Identifier: Apache-2.0

{
  pkgs,
  inputs,
  system,
}: {
  packages = with pkgs; [
    go-ethereum
    solc_0_8_28
    (inputs.solc.mkDefault pkgs solc_0_8_28)
    # py-evm (a slither dependency) does not support python 3.14, which is
    # the default python3 in nixpkgs-unstable; pin slither to python 3.13.
    (python313Packages.toPythonApplication python313Packages.slither-analyzer)
    inputs.natlint.packages.${system}.default
  ];
}
