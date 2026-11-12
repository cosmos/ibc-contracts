# SPDX-License-Identifier: Apache-2.0

{
  pkgs,
  inputs,
  system,
}: {
  packages = with pkgs; [
    # Provides abigen; keep the abigen pin in .github/workflows/abigen.yaml in sync
    # with this version, since abigen releases change the generated bindings.
    go-ethereum
    solc_0_8_37
    (inputs.solc.mkDefault pkgs solc_0_8_37)
    # py-evm (a slither dependency) does not support python 3.14, which is
    # the default python3 in nixpkgs-unstable; pin slither to python 3.13.
    (python313Packages.toPythonApplication python313Packages.slither-analyzer)
    inputs.natlint.packages.${system}.default
  ];
}
