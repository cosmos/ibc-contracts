// SPDX-License-Identifier: Apache-2.0

import { readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

type Mode = "update" | "check";
type Snapshot = Readonly<Record<string, number>>;

const BENCHMARK_DIR = import.meta.dir;
const PROJECT_ROOT = resolve(BENCHMARK_DIR, "..");
const SNAPSHOT_DIR = join(BENCHMARK_DIR, "snapshots");
const README_PATH = join(BENCHMARK_DIR, "README.md");

export const SP1_KEYS = [
  "erc20.groth16.1.ack.calldata",
  "erc20.groth16.1.ack.gas",
  "erc20.groth16.1.recv.calldata",
  "erc20.groth16.1.recv.gas",
  "erc20.groth16.1.send.calldata",
  "erc20.groth16.1.send.gas",
  "erc20.groth16.25.ack.calldata",
  "erc20.groth16.25.ack.gas",
  "erc20.groth16.25.recv.calldata",
  "erc20.groth16.25.recv.gas",
  "erc20.groth16.25.send.calldata",
  "erc20.groth16.25.send.gas",
  "erc20.groth16.50.ack.calldata",
  "erc20.groth16.50.ack.gas",
  "erc20.groth16.50.recv.calldata",
  "erc20.groth16.50.recv.gas",
  "erc20.groth16.50.send.calldata",
  "erc20.groth16.50.send.gas",
  "erc20.plonk.1.ack.calldata",
  "erc20.plonk.1.ack.gas",
  "erc20.plonk.1.recv.calldata",
  "erc20.plonk.1.recv.gas",
  "erc20.plonk.1.send.calldata",
  "erc20.plonk.1.send.gas",
  "erc20.plonk.50.ack.calldata",
  "erc20.plonk.50.ack.gas",
  "erc20.plonk.50.recv.calldata",
  "erc20.plonk.50.recv.gas",
  "erc20.plonk.50.send.calldata",
  "erc20.plonk.50.send.gas",
  "native.groth16.1.recv.calldata",
  "native.groth16.1.recv.gas",
  "native.plonk.1.recv.calldata",
  "native.plonk.1.recv.gas",
  "timeout.groth16.1.timeout.calldata",
  "timeout.groth16.1.timeout.gas",
  "timeout.groth16.1.send.calldata",
  "timeout.groth16.1.send.gas",
  "timeout.plonk.1.timeout.calldata",
  "timeout.plonk.1.timeout.gas",
  "timeout.plonk.1.send.calldata",
  "timeout.plonk.1.send.gas",
] as const;

export const BESU_QBFT_KEYS = [
  "update.adjacent.calldata",
  "update.adjacent.gas",
  "update.non_adjacent.calldata",
  "update.non_adjacent.gas",
  "verify_membership.calldata",
  "verify_membership.gas",
] as const;

export function validateSnapshot(
  raw: unknown,
  expectedKeys: readonly string[],
  name: string,
): Snapshot {
  if (raw === null || Array.isArray(raw) || typeof raw !== "object") {
    throw new Error(`${name} must be a JSON object`);
  }

  const entries = Object.entries(raw);
  const actualKeys = entries.map(([key]) => key).sort();
  const expected = [...expectedKeys].sort();
  const missing = expected.filter((key) => !actualKeys.includes(key));
  const unexpected = actualKeys.filter((key) => !expected.includes(key));
  if (missing.length !== 0 || unexpected.length !== 0) {
    throw new Error(
      `${name} schema mismatch; missing: ${missing.join(", ") || "none"}; unexpected: ${unexpected.join(", ") || "none"}`,
    );
  }

  return Object.fromEntries(
    entries.map(([key, value]) => {
      if (typeof value !== "string" || !/^\d+$/.test(value)) {
        throw new Error(
          `${name}.${key} must be an unsigned integer encoded as a string`,
        );
      }
      const parsed = Number(value);
      if (!Number.isSafeInteger(parsed)) {
        throw new Error(
          `${name}.${key} exceeds JavaScript's safe integer range`,
        );
      }
      return [key, parsed];
    }),
  );
}

export function formatInteger(value: number): string {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`cannot format non-negative integer: ${value}`);
  }
  return value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

export function averagePerPacket(
  totalGas: number,
  packetCount: number,
): number {
  if (
    !Number.isSafeInteger(totalGas) ||
    totalGas < 0 ||
    !Number.isSafeInteger(packetCount) ||
    packetCount <= 0
  ) {
    throw new Error("gas and packet count must be valid integers");
  }
  return Math.floor(totalGas / packetCount);
}

function value(snapshot: Snapshot, key: string): string {
  const result = snapshot[key];
  if (result === undefined) {
    throw new Error(`missing validated benchmark value: ${key}`);
  }
  return formatInteger(result);
}

function singlePacketRow(
  sp1: Snapshot,
  operation: string,
  grothGas: string,
  plonkGas: string,
  grothCalldata?: string,
  plonkCalldata?: string,
): string {
  return `| ${operation} | ${value(sp1, grothGas)} | ${value(sp1, plonkGas)} | ${grothCalldata ? value(sp1, grothCalldata) : "—"} | ${plonkCalldata ? value(sp1, plonkCalldata) : "—"} |`;
}

function aggregatedRow(
  sp1: Snapshot,
  proof: string,
  packets: number,
  operation: string,
  gasKey: string,
  calldataKey: string,
): string {
  const totalGas = sp1[gasKey];
  if (totalGas === undefined) {
    throw new Error(`missing validated benchmark value: ${gasKey}`);
  }
  return `| ${proof} | ${packets} | ${operation} | ${formatInteger(totalGas)} | ${formatInteger(averagePerPacket(totalGas, packets))} | ${value(sp1, calldataKey)} |`;
}

export function renderReadme(sp1: Snapshot, besu: Snapshot): string {
  const lines = [
    "<!-- This file is generated by `just solidity::test-benchmark`. Do not edit it manually. -->",
    "",
    "# Solidity Benchmarks",
    "",
    "These values are generated from deterministic Foundry tests and checked into the repository so benchmark changes are visible in GitHub diffs. Gas matches transaction receipt `gasUsed`: the post-refund total includes intrinsic gas, calldata, proxy routing, and execution.",
    "",
    "The benchmark profile uses Solidity 0.8.28, the Cancun EVM, IR compilation, 10,000 optimizer runs, and the fixtures committed under `test/`.",
    "",
    "Regenerate the JSON snapshots and this document from the repository root:",
    "",
    "```sh",
    "just solidity::test-benchmark",
    "```",
    "",
    "## SP1 Tendermint end-to-end packet benchmarks",
    "",
    "Each measurement runs as an isolated transaction. Relay operations include the complete router multicall, SP1 proof verification, packet handling, and application callbacks. The ERC20 send assumes the token allowance is already set.",
    "",
    "The send value is the average of 50 separate transactions, including the more expensive first transfer that deploys the escrow contract.",
    "",
    "### Packet operations",
    "",
    "| Operation | Groth16 gas | Plonk gas | Groth16 calldata | Plonk calldata |",
    "| --- | ---: | ---: | ---: | ---: |",
    singlePacketRow(
      sp1,
      "Send ERC20 (50-transaction average)",
      "erc20.groth16.50.send.gas",
      "erc20.plonk.50.send.gas",
      "erc20.groth16.50.send.calldata",
      "erc20.plonk.50.send.calldata",
    ),
    singlePacketRow(
      sp1,
      "Acknowledge ERC20 end-to-end",
      "erc20.groth16.1.ack.gas",
      "erc20.plonk.1.ack.gas",
      "erc20.groth16.1.ack.calldata",
      "erc20.plonk.1.ack.calldata",
    ),
    singlePacketRow(
      sp1,
      "Receive returning ERC20 end-to-end",
      "erc20.groth16.1.recv.gas",
      "erc20.plonk.1.recv.gas",
      "erc20.groth16.1.recv.calldata",
      "erc20.plonk.1.recv.calldata",
    ),
    singlePacketRow(
      sp1,
      "Receive new Cosmos token end-to-end",
      "native.groth16.1.recv.gas",
      "native.plonk.1.recv.gas",
      "native.groth16.1.recv.calldata",
      "native.plonk.1.recv.calldata",
    ),
    singlePacketRow(
      sp1,
      "Timeout ERC20 end-to-end",
      "timeout.groth16.1.timeout.gas",
      "timeout.plonk.1.timeout.gas",
      "timeout.groth16.1.timeout.calldata",
      "timeout.plonk.1.timeout.calldata",
    ),
    "",
    "### Batched router multicalls",
    "",
    "| Proof | Packets | Operation | Total gas | Average gas / packet | Calldata bytes |",
    "| --- | ---: | --- | ---: | ---: | ---: |",
    aggregatedRow(
      sp1,
      "Groth16",
      25,
      "Acknowledge ERC20 end-to-end",
      "erc20.groth16.25.ack.gas",
      "erc20.groth16.25.ack.calldata",
    ),
    aggregatedRow(
      sp1,
      "Groth16",
      25,
      "Receive returning ERC20 end-to-end",
      "erc20.groth16.25.recv.gas",
      "erc20.groth16.25.recv.calldata",
    ),
    aggregatedRow(
      sp1,
      "Groth16",
      50,
      "Acknowledge ERC20 end-to-end",
      "erc20.groth16.50.ack.gas",
      "erc20.groth16.50.ack.calldata",
    ),
    aggregatedRow(
      sp1,
      "Groth16",
      50,
      "Receive returning ERC20 end-to-end",
      "erc20.groth16.50.recv.gas",
      "erc20.groth16.50.recv.calldata",
    ),
    aggregatedRow(
      sp1,
      "Plonk",
      50,
      "Acknowledge ERC20 end-to-end",
      "erc20.plonk.50.ack.gas",
      "erc20.plonk.50.ack.calldata",
    ),
    aggregatedRow(
      sp1,
      "Plonk",
      50,
      "Receive returning ERC20 end-to-end",
      "erc20.plonk.50.recv.gas",
      "erc20.plonk.50.recv.calldata",
    ),
    "",
    "## Besu QBFT light-client benchmarks",
    "",
    "These isolated transactions use the live Besu QBFT header and proof fixture in `test/besu-bft/fixtures/qbft.json`. They measure direct light-client calls and are not directly comparable with the SP1 packet operations above.",
    "",
    "| Operation | Gas | ABI calldata bytes |",
    "| --- | ---: | ---: |",
    `| Adjacent client update | ${value(besu, "update.adjacent.gas")} | ${value(besu, "update.adjacent.calldata")} |`,
    `| Non-adjacent client update | ${value(besu, "update.non_adjacent.gas")} | ${value(besu, "update.non_adjacent.calldata")} |`,
    `| Membership verification | ${value(besu, "verify_membership.gas")} | ${value(besu, "verify_membership.calldata")} |`,
    "",
  ];

  return lines.join("\n");
}

export function assertGeneratedContent(
  actual: string,
  expected: string,
  path: string,
): void {
  if (actual !== expected) {
    throw new Error(
      `${path} is stale; run \`just solidity::test-benchmark\` and commit the result`,
    );
  }
}

async function readSnapshot(
  fileName: string,
  keys: readonly string[],
): Promise<Snapshot> {
  const raw = JSON.parse(await readFile(join(SNAPSHOT_DIR, fileName), "utf8"));
  return validateSnapshot(raw, keys, fileName);
}

async function runForge(mode: Mode): Promise<void> {
  const args = [
    "forge",
    "test",
    "--match-path",
    "test/benchmarks/*.t.sol",
    "--match-test",
    "^testBenchmark_",
    "--isolate",
    "--gas-snapshot-check",
    mode === "check" ? "true" : "false",
    "--gas-snapshot-emit",
    mode === "update" ? "true" : "false",
  ];
  const process = Bun.spawn(args, {
    cwd: PROJECT_ROOT,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await process.exited;
  if (exitCode !== 0) {
    throw new Error(`forge benchmark tests failed with exit code ${exitCode}`);
  }
}

async function main(): Promise<void> {
  const mode = process.argv[2];
  if (mode !== "update" && mode !== "check") {
    throw new Error("usage: bun benchmarks/generate.ts <update|check>");
  }

  await runForge(mode);
  const [sp1, besu] = await Promise.all([
    readSnapshot("SP1E2E.json", SP1_KEYS),
    readSnapshot("BesuQBFT.json", BESU_QBFT_KEYS),
  ]);
  const expectedReadme = renderReadme(sp1, besu);

  if (mode === "update") {
    await writeFile(README_PATH, expectedReadme);
    console.log(`Updated ${README_PATH}`);
    return;
  }

  assertGeneratedContent(
    await readFile(README_PATH, "utf8"),
    expectedReadme,
    README_PATH,
  );
  console.log("Benchmark snapshots and README are up to date");
}

if (import.meta.main) {
  await main();
}
