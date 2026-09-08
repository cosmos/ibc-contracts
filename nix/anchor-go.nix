# SPDX-License-Identifier: Apache-2.0

{pkgs}:
pkgs.buildGoModule rec {
  pname = "anchor-go";
  version = "1.0.0";

  src = pkgs.fetchFromGitHub {
    owner = "gagliardetto";
    repo = "anchor-go";
    rev = "v${version}";
    sha256 = "sha256-Q7ZRuHvWTkDZl2D2AY/LdqdtFbrK4Rsiq+BhPr469YU=";
  };

  vendorHash = "sha256-C8Ne0aHe3GW130tasJ7+x4eq8Yp2zo2GK0AbWEi93dE=";

  patches = [
    ./anchor-go-fix-complex-enum.patch
    # https://github.com/gagliardetto/anchor-go/pull/52
    ./anchor-go-fix-no-args-discriminator.patch
  ];

  doCheck = false;

  meta = with pkgs.lib; {
    description = "Golang Anchor client";
    homepage = "https://github.com/gagliardetto/anchor-go";
    license = licenses.mit;
    maintainers = with maintainers; [vaporif];
  };
}
