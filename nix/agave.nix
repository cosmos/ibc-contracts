# SPDX-License-Identifier: Apache-2.0

{
  lib,
  stdenv,
  fetchFromGitHub,
  symlinkJoin,
  fetchurl,
  rustPlatform,
  pkg-config,
  openssl,
  zlib,
  protobuf,
  perl,
  hidapi,
  udev,
  llvmPackages,
  rust-bin,
  writeShellScriptBin,
  anchor,
  jq,
  solanaPkgs ? [
    "cargo-build-sbf"
    "cargo-test-sbf"
    "solana"
    "solana-bench-tps"
    "solana-faucet"
    "solana-gossip"
    "solana-keygen"
    "solana-log-analyzer"
    "solana-net-shaper"
    "solana-test-validator"
    "solana-genesis"
    "agave-ledger-tool"
    "agave-install"
    "agave-validator"
  ],
}: let
  inherit (lib) optionals;
  inherit (stdenv) hostPlatform isLinux;

  versions = {
    agave = "2.3.13";
    platformTools = "v1.48";
  };

  # Create nightly toolchain from rust-bin (used for IDL generation)
  rustNightly = rust-bin.nightly.latest.default.override {
    extensions = ["rust-src"];
  };

  platformConfig = {
    x86_64-darwin = {
      archive = "platform-tools-osx-x86_64.tar.bz2";
      sha256 = "sha256-vLTtCmUkxxkd8KKQa8qpQ7kb5S52EI/DVllgtu8zM2I=";
    };
    aarch64-darwin = {
      archive = "platform-tools-osx-aarch64.tar.bz2";
      sha256 = "sha256-eZ5M/O444icVXIP7IpT5b5SoQ9QuAcA1n7cSjiIW0t0=";
    };
    x86_64-linux = {
      archive = "platform-tools-linux-x86_64.tar.bz2";
      sha256 = "sha256-qdMVf5N9X2+vQyGjWoA14PgnEUpmOwFQ20kuiT7CdZc=";
    };
    aarch64-linux = {
      archive = "platform-tools-linux-aarch64.tar.bz2";
      sha256 = "sha256-rsYCIiL3ueJHkDZkhLzGz59mljd7uY9UHIhp4vMecPI=";
    };
  };

  currentPlatform = platformConfig.${hostPlatform.system} or
    (throw "Unsupported platform: ${hostPlatform.system}");

  platformToolsArchive = fetchurl {
    url = "https://github.com/anza-xyz/platform-tools/releases/download/${versions.platformTools}/${currentPlatform.archive}";
    inherit (currentPlatform) sha256;
  };

  # Download SBF SDK archive from Agave releases
  sbfSdkArchive = fetchurl {
    url = "https://github.com/anza-xyz/agave/releases/download/v${versions.agave}/sbf-sdk.tar.bz2";
    sha256 = "sha256-zdGtFHxj/I4ID3RN3BNx27LakxzhwOuvSZpVb3M93YM=";
  };

  # SBF SDK derivation
  sbfSdk = stdenv.mkDerivation {
    pname = "sbf-sdk";
    version = versions.agave;

    src = sbfSdkArchive;

    unpackPhase = ''
      mkdir -p $out
      tar -xjf $src -C $out

      # Create symlink to platform tools
      mkdir -p $out/dependencies
      ln -s ${platformTools} $out/dependencies/platform-tools

      # Extract scripts from agave/platform-tools-sdk/sbf/scripts/
      if [ -d "${agave.src}/platform-tools-sdk/sbf/scripts" ]; then
        mkdir -p $out/scripts
        cp -r ${agave.src}/platform-tools-sdk/sbf/scripts/* $out/scripts/
        chmod +x $out/scripts/*.sh 2>/dev/null || true
      fi

      # Create env.sh at the root to fix strip.sh script path
      if [ -f "$out/sbf-sdk/env.sh" ]; then
        ln -s $out/sbf-sdk/env.sh $out/env.sh
      fi
    '';

    meta = with lib; {
      description = "Solana BPF SDK for building on-chain programs";
      homepage = "https://github.com/anza-xyz/agave";
      license = licenses.asl20;
      maintainers = with maintainers; [vaporif];
      platforms = platforms.unix;
    };
  };

  platformTools = stdenv.mkDerivation {
    pname = "platformTools";
    version = versions.platformTools;

    src = platformToolsArchive;

    unpackPhase = ''
      mkdir -p $out
      tar -xjf $src -C $out

      # ldb-argdumper in this package will point to a dangling link
      # we're only building not debugging so safe to just ignore
      find $out -type l ! -exec test -e {} \; -delete 2>/dev/null || true
    '';

    meta = with lib; {
      description = "Solana platform tools for building on-chain programs";
      homepage = "https://github.com/anza-xyz/platformTools";
      license = licenses.asl20;
      maintainers = with maintainers; [vaporif];
      platforms = platforms.unix;
    };
  };

  # Use Rust stable version that's compatible with Agave version (from rust-toolchain.toml)
  rustForAgave = rust-bin.stable."1.86.0".default.override {
    extensions = ["rust-src"];
  };

  agave =
    rustPlatform.buildRustPackage.override {
      rustc = rustForAgave;
      cargo = rustForAgave;
    } {
      pname = "agave";
      version = versions.agave;

      src = fetchFromGitHub {
        owner = "anza-xyz";
        repo = "agave";
        rev = "v${versions.agave}";
        hash = "sha256-RSucqvbshaaby4fALhAQJtZztwsRdA+X7yRnoBxQvsg=";
        fetchSubmodules = true;
      };

      cargoHash = "sha256-yTS++bUu+4wmbXXZkU4eDq4sGNzls1euptJoY6OYZOM=";

      cargoBuildFlags = map (n: "--bin=${n}") solanaPkgs;

      nativeBuildInputs = [
        pkg-config
        protobuf
        perl
        llvmPackages.clang
      ];

      buildInputs =
        [
          openssl
          zlib
          llvmPackages.libclang.lib
        ]
        ++ optionals isLinux [
          hidapi
          udev
        ];

      LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

      # TODO: This can be removed once https://github.com/facebook/rocksdb/issues/13365 is resolved
      CXXFLAGS = lib.optionalString isLinux "-include cstdint";

      BINDGEN_EXTRA_CLANG_ARGS = toString (
        [
          "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion llvmPackages.clang}/include"
        ]
        ++ optionals isLinux [
          "-isystem ${stdenv.cc.libc.dev}/include"
        ]
        ++ optionals hostPlatform.isDarwin [
          "-isystem ${stdenv.cc.libc}/include"
        ]
      );

      postPatch = ''
        substituteInPlace scripts/cargo-install-all.sh \
          --replace-fail './fetch-perf-libs.sh' 'echo "Skipping fetch-perf-libs in Nix build"' \
          --replace-fail '"$cargo" $maybeRustVersion install' 'echo "Skipping cargo install"'
      '';

      doCheck = false;

      meta = with lib; {
        description = "Solana cli and programs";
        homepage = "https://github.com/anza-xyz/agave";
        license = licenses.asl20;
        maintainers = with maintainers; [vaporif];
        platforms = platforms.unix;
      };
    };

  # Anchor-nix wrapper script
  #
  # Why we need this wrapper:
  # 1. Anchor CLI requires different Rust toolchains for different operations:
  #    - Building Solana programs: Requires the specific Solana/Agave Rust toolchain
  #      that is forked and maintained by Solana
  #    - Generating IDLs: Requires Rust nightly
  #    - Tests: Requires Rust nightly
  # 2. The wrapper intelligently switches between toolchains:
  #    - Strips existing Rust paths from PATH to avoid conflicts
  #    - Sets up the correct toolchain for each operation
  #    - No need to add rustup shims
  #    - Handles the complexity of toolchain management transparently
  #
  anchorNix = writeShellScriptBin "anchor-nix" ''
        #!${stdenv.shell}
        set -euo pipefail

        readonly REAL_ANCHOR="${anchor}/bin/anchor"
        export SBF_SDK_PATH="${sbfSdk}"

        clean_rust_from_path() {
          echo "$PATH" | tr ':' '\n' | \
            grep -v "rust-bin" | \
            grep -v ".cargo/bin" | \
            grep -v "rustup" | \
            tr '\n' ':'
        }

        setup_solana() {
          export PATH=$(clean_rust_from_path)

          export PATH="${platformTools}/rust/bin:$PATH"
          export RUSTC="${platformTools}/rust/bin/rustc"
          export CARGO="${platformTools}/rust/bin/cargo"
        }

        setup_nightly() {
          export PATH=$(clean_rust_from_path | sed "s|${platformTools}||g")

          unset RUSTC CARGO || true

          export PATH="${rustNightly}/bin:$PATH"

          # Make SBF target specs available to nightly rust for anchor idl build
          # anchor idl build runs cargo test which needs access to sbf-solana-solana target
          export RUST_TARGET_PATH="${platformTools}/rust/lib/rustlib"
        }

        has_idl_build_feature() {
          find programs -name "Cargo.toml" -type f 2>/dev/null | \
            xargs grep -l "idl-build" 2>/dev/null | \
            head -n1
        }

        run_build() {
          local extra_args=("$@")

          echo "🔨 Building Solana program with solana toolchain setup..."
          echo "📦 Building program with Solana/Agave toolchain..."

          setup_solana

          # Parse arguments
          local skip_idl=false
          local cargo_args=()
          local specific_package=""

          for arg in "''${extra_args[@]}"; do
            if [[ "$arg" == "--no-idl" ]]; then
              skip_idl=true
            elif [[ "$arg" != "--" ]]; then
              cargo_args+=("$arg")
            fi
          done

          # Check for -p or --manifest-path flags
          local manifest_path=""
          for ((i=0; i<''${#cargo_args[@]}; i++)); do
            if [[ "''${cargo_args[$i]}" == "-p" && $((i+1)) -lt ''${#cargo_args[@]} ]]; then
              specific_package="''${cargo_args[$((i+1))]}"
              echo "🎯 Building package: $specific_package"
              break
            elif [[ "''${cargo_args[$i]}" == "--manifest-path" && $((i+1)) -lt ''${#cargo_args[@]} ]]; then
              manifest_path="''${cargo_args[$((i+1))]}"
              # Derive specific_package from manifest path for IDL filtering
              local manifest_dir
              manifest_dir="$(dirname "$manifest_path")"
              specific_package="$(basename "$manifest_dir")"
              echo "🎯 Building from manifest: $manifest_path"
              break
            fi
          done

          # Build the program(s)
          if [ -n "$manifest_path" ]; then
            # Build using the provided manifest path directly
            if ! cargo build-sbf --manifest-path "$manifest_path" --no-rustup-override --skip-tools-install; then
              echo "❌ Program build failed"
              return 1
            fi
          elif [ -n "$specific_package" ]; then
            # Build specific package by finding its directory
            echo "   Building only $specific_package (skipping other programs)..."

            local program_dir=""
            local hyphenated="''${specific_package//_/-}"
            for dir in programs/*/; do
              local dirname
              dirname="$(basename "$dir")"
              if [ "$dirname" = "$specific_package" ] || [ "$dirname" = "$hyphenated" ]; then
                program_dir="$dir"
                break
              fi
            done

            if [ -z "$program_dir" ]; then
              echo "❌ Program directory not found for: $specific_package"
              return 1
            fi

            if ! cargo build-sbf --manifest-path "''${program_dir}Cargo.toml" --no-rustup-override --skip-tools-install; then
              echo "❌ Program build failed"
              return 1
            fi
          else
            # Build all programs using anchor
            if ! "$REAL_ANCHOR" build --no-idl -- --no-rustup-override --skip-tools-install; then
              echo "❌ Program build failed"
              return 1
            fi
          fi

          # Skip IDL generation if requested
          if [ "$skip_idl" = true ]; then
            echo "⏭️  Skipping IDL generation (--no-idl flag)"
            return 0
          fi

          if cargo_toml=$(has_idl_build_feature); then
            echo "📝 Generating IDL with nightly toolchain..."

            setup_nightly

            echo "📤 Extracting IDL files..."
            mkdir -p target/idl

            local idl_success=0
            local idl_failed=0
            local idl_total=0

            # Extract IDL from each program using cargo test with idl-build feature
            for program_dir in programs/*/; do
              program_name=$(basename "$program_dir")

              # Skip if building specific package and this isn't it
              if [ -n "$specific_package" ] && [ "$program_name" != "$specific_package" ]; then
                continue
              fi

              has_idl_build=false
              if [ -f "$program_dir/Cargo.toml" ]; then
                if grep -q "idl-build" "$program_dir/Cargo.toml" 2>/dev/null || true; then
                  has_idl_build=true
                fi
              fi

              if [ "$has_idl_build" = "true" ]; then
                ((idl_total++)) || true
                echo "   [✓] $program_name has idl-build feature ($idl_total)..."

                # Set required environment variables for IDL generation (from idl/src/build.rs:156-169)
                export ANCHOR_IDL_BUILD_PROGRAM_PATH="$program_dir"
                export ANCHOR_IDL_BUILD_RESOLUTION="TRUE"
                export ANCHOR_IDL_BUILD_NO_DOCS="FALSE"
                export ANCHOR_IDL_BUILD_SKIP_LINT="TRUE"
                export RUSTFLAGS="-A warnings"

                # Build this specific program with idl-build feature first
                echo "      Building with idl-build feature..."
                build_output=$(cargo build \
                  --manifest-path "$program_dir/Cargo.toml" \
                  --features idl-build \
                  --lib 2>&1) || build_exit=$?
                build_exit=''${build_exit:-0}

                if [ "$build_exit" -ne 0 ]; then
                  echo "   ⚠️  Build failed for $program_name, skipping IDL extraction"
                  echo "   Build output (last 10 lines):" >&2
                  echo "$build_output" | tail -10 >&2
                  ((idl_failed++)) || true
                  unset ANCHOR_IDL_BUILD_PROGRAM_PATH ANCHOR_IDL_BUILD_RESOLUTION
                  unset ANCHOR_IDL_BUILD_NO_DOCS ANCHOR_IDL_BUILD_SKIP_LINT RUSTFLAGS
                  continue
                fi

                echo "      Build succeeded, now extracting IDL..."
                temp_output="/tmp/idl_$program_name.txt"

                set +e  # Temporarily disable exit on error for test command
                cargo test \
                  --manifest-path "$program_dir/Cargo.toml" \
                  --features idl-build \
                  --lib \
                  __anchor_private_print_idl \
                  -- \
                  --show-output \
                  --quiet \
                  --test-threads=1 > "$temp_output" 2>&1
                test_exit=$?
                set -e  # Re-enable exit on error

                if [ "$test_exit" -eq 0 ]; then
                  # Extract IDL JSON from program section (see idl/src/build.rs:202-280)
                  idl_json=$(cat "$temp_output" | awk '
                    BEGIN { in_program=0; program="" }
                    /--- IDL begin program ---/ { in_program=1; next }
                    /--- IDL end program ---/ { in_program=0; next }
                    in_program { program = program $0 "\n" }
                    END { printf "%s", program }
                  ')

                  # Extract events from "--- IDL begin event ---" / "--- IDL end event ---" sections
                  events_json=$(cat "$temp_output" | awk '
                    BEGIN { in_event=0; events="[" }
                    /--- IDL begin event ---/ { in_event=1; event_content=""; next }
                    /--- IDL end event ---/ {
                      in_event=0
                      if (events != "[") events = events ","
                      events = events event_content
                      next
                    }
                    in_event { event_content = event_content $0 }
                    END { printf "%s]", events }
                  ')

                  if [ -n "$idl_json" ] && [ "$(echo "$idl_json" | tr -d '[:space:]')" != "" ]; then
                    idl_filename=$(echo "$program_name" | tr '-' '_')

                    # Merge events into the IDL if we found any
                    if [ -n "$events_json" ] && [ "$events_json" != "[]" ]; then
                      # Extract event entries and types from each event block, then merge into IDL
                      idl_json=$(echo "$idl_json" | ${jq}/bin/jq --argjson events "$events_json" '
                        # Collect all event definitions
                        ($events | map(.event)) as $event_defs |
                        # Collect all types from events
                        ($events | map(.types) | flatten) as $event_types |
                        # Merge into IDL
                        .events = $event_defs |
                        .types = ((.types // []) + $event_types | unique_by(.name))
                      ')
                      echo "    → Merged $(echo "$events_json" | ${jq}/bin/jq 'length') event(s) into IDL"
                    fi

                    # Get program ID from keypair if it exists
                    keypair_file="target/deploy/''${idl_filename}-keypair.json"
                    if [ -f "$keypair_file" ]; then
                      program_id=$(${agave}/bin/solana-keygen pubkey "$keypair_file")
                      # Add the address field to the IDL JSON
                      idl_json=$(echo "$idl_json" | ${jq}/bin/jq --arg addr "$program_id" '. + {address: $addr}')
                      echo "    → Added program ID: $program_id"
                    fi

                    echo "$idl_json" > "target/idl/''${idl_filename}.json"
                    echo "    ✓ Generated target/idl/''${idl_filename}.json"
                    ((idl_success++)) || true
                    rm -f "$temp_output"
                  else
                    echo "   ⚠️  Failed to extract IDL for $program_name (no program section found)"
                    echo "   [DEBUG] First 30 lines of test output:"
                    head -30 "$temp_output" >&2
                    echo "   [DEBUG] Searching for IDL markers:"
                    grep -n "IDL" "$temp_output" >&2 || echo "   No IDL markers found" >&2
                    ((idl_failed++)) || true
                    rm -f "$temp_output"
                  fi
                else
                  echo "   ❌ IDL test failed for $program_name (exit code: $test_exit)"
                  echo "   [DEBUG] Last 30 lines of output:"
                  tail -30 "$temp_output" >&2
                  ((idl_failed++)) || true
                  rm -f "$temp_output"
                fi

                # Clean up env vars
                unset ANCHOR_IDL_BUILD_PROGRAM_PATH ANCHOR_IDL_BUILD_RESOLUTION
                unset ANCHOR_IDL_BUILD_NO_DOCS ANCHOR_IDL_BUILD_SKIP_LINT RUSTFLAGS
              else
                echo "   [✗] $program_name does not have idl-build feature, skipping"
              fi
            done

            # Report results and return appropriate exit code
            if [ "$idl_success" -gt 0 ] && [ "$idl_failed" -eq 0 ]; then
              echo "✅ Build complete: generated $idl_success IDL file(s)"
            elif [ "$idl_success" -gt 0 ] && [ "$idl_failed" -gt 0 ]; then
              echo "⚠️  Build complete: generated $idl_success IDL file(s), $idl_failed failed"
              return 1
            elif [ "$idl_failed" -gt 0 ]; then
              echo "❌ Build complete but all IDL generation failed ($idl_failed program(s))"
              return 1
            else
              echo "ℹ️  No programs with idl-build feature found"
            fi
          else
            echo "ℹ️  Skipping IDL generation (no idl-build feature found in Cargo.toml)"
            echo "✅ Build complete: program built with Solana toolchain"
          fi
        }

        # Function to run tests
        run_test() {
          local extra_args=("$@")

          echo "🧪 Testing Solana program..."

          if ! run_build "''${extra_args[@]}"; then
            return 1
          fi

          setup_nightly

          echo "🧪 Running tests with nightly toolchain..."
          "$REAL_ANCHOR" test --skip-build "''${extra_args[@]}"
        }

        # Function to run unit tests with cargo test
        run_unit_test() {
          local extra_args=("$@")

          echo "🧪 Running unit tests..."

          if ! run_build; then
            return 1
          fi

          setup_nightly

          echo "🧪 Running cargo test with nightly toolchain..."
          cargo test "''${extra_args[@]}"
        }

        # Main command dispatcher
        case "''${1:-}" in
          build)
            shift
            run_build "$@"
            ;;

          test)
            shift
            run_test "$@"
            ;;

          unit-test)
            shift
            run_unit_test "$@"
            ;;

          keys)
            # Pass through to real anchor for keys command (sync, list, etc.)
            "$REAL_ANCHOR" "$@"
            ;;

          deploy)
            # Pass through to real anchor for deploy command
            "$REAL_ANCHOR" "$@"
            ;;

          *)
            cat <<EOF
    anchor-nix: Anchor wrapper for Nix environments

    Usage:
      anchor-nix build [options]      - Build program with Solana toolchain, generate IDL with nightly
      anchor-nix test [options]       - Build and run anchor client tests
      anchor-nix unit-test [options]  - Build program then run cargo test
      anchor-nix keys [subcommand]    - Manage program keypairs (sync, list, etc.)
      anchor-nix deploy [options]     - Deploy programs to specified cluster

    This wrapper automatically handles toolchain switching to provide:
      - Fast, deterministic builds with Solana/Agave toolchain
      - IDL generation with Rust nightly toolchain

    EOF
            exit 1
            ;;
        esac
  '';
in
  symlinkJoin {
    name = "agave-with-toolchain-${versions.agave}";
    paths = [agave anchorNix anchor];

    passthru = {
      inherit agave rustNightly;
    };

    meta =
      agave.meta
      // {
        description = "Solana programs & tooling with Anchor wrapper";
        mainProgram = "anchor-nix";
      };
  }
