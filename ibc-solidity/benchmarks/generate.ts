// SPDX-License-Identifier: Apache-2.0

import { readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

type Mode = "update" | "check";
type Snapshot = Readonly<Record<string, number>>;
type GasReportFunction = Readonly<{
  calls: number;
  min: number;
  mean: number;
  median: number;
  max: number;
}>;

const BENCHMARK_DIR = import.meta.dir;
const PROJECT_ROOT = resolve(BENCHMARK_DIR, "..");
const SNAPSHOT_DIR = join(BENCHMARK_DIR, "snapshots");
const README_PATH = join(BENCHMARK_DIR, "README.md");
const SP1_GAS_REPORT_FILE = "SP1GasReport.json";
const GAS_REPORT_STATS = ["calls", "min", "mean", "median", "max"] as const;

const GAS_REPORT_FUNCTIONS = {
  send_transfer: {
    contract: "contracts/ICS20Transfer.sol:ICS20Transfer",
    signature:
      "sendTransfer((address,uint256,string,string,string,uint64,string))",
  },
  ack_packet: {
    contract: "contracts/ICS26Router.sol:ICS26Router",
    signature:
      "ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)))",
  },
  recv_packet: {
    contract: "contracts/ICS26Router.sol:ICS26Router",
    signature:
      "recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)))",
  },
} as const;

export const SP1_GAS_REPORT_KEYS = ["groth16", "plonk"].flatMap((proof) =>
  Object.keys(GAS_REPORT_FUNCTIONS).flatMap((operation) =>
    GAS_REPORT_STATS.map((stat) => `erc20.${proof}.50.${operation}.${stat}`),
  ),
);

export const SP1_KEYS = [
  "erc20.groth16.1.ack.calldata",
  "erc20.groth16.1.ack.gas",
  "erc20.groth16.1.recv.calldata",
  "erc20.groth16.1.recv.gas",
  "erc20.groth16.25.ack.calldata",
  "erc20.groth16.25.ack.gas",
  "erc20.groth16.25.recv.calldata",
  "erc20.groth16.25.recv.gas",
  "erc20.groth16.50.ack.calldata",
  "erc20.groth16.50.ack.gas",
  "erc20.groth16.50.recv.calldata",
  "erc20.groth16.50.recv.gas",
  "erc20.plonk.1.ack.calldata",
  "erc20.plonk.1.ack.gas",
  "erc20.plonk.1.recv.calldata",
  "erc20.plonk.1.recv.gas",
  "erc20.plonk.50.ack.calldata",
  "erc20.plonk.50.ack.gas",
  "erc20.plonk.50.recv.calldata",
  "erc20.plonk.50.recv.gas",
  "native.groth16.1.recv.calldata",
  "native.groth16.1.recv.gas",
  "native.plonk.1.recv.calldata",
  "native.plonk.1.recv.gas",
  "timeout.groth16.1.timeout.calldata",
  "timeout.groth16.1.timeout.gas",
  "timeout.plonk.1.timeout.calldata",
  "timeout.plonk.1.timeout.gas",
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

function requireRecord(raw: unknown, name: string): Record<string, unknown> {
  if (raw === null || Array.isArray(raw) || typeof raw !== "object") {
    throw new Error(`${name} must be an object`);
  }
  return raw as Record<string, unknown>;
}

function parseGasReportFunction(
  raw: unknown,
  contractName: string,
  signature: string,
): GasReportFunction {
  if (!Array.isArray(raw)) {
    throw new Error("Forge gas report must be a JSON array");
  }

  const contract = raw
    .map((entry, index) => requireRecord(entry, `gas report entry ${index}`))
    .find((entry) => entry.contract === contractName);
  if (contract === undefined) {
    throw new Error(`missing gas report contract: ${contractName}`);
  }

  const functions = requireRecord(
    contract.functions,
    `${contractName}.functions`,
  );
  const report = requireRecord(
    functions[signature],
    `${contractName}.${signature}`,
  );

  return Object.fromEntries(
    GAS_REPORT_STATS.map((stat) => {
      const result = report[stat];
      if (!Number.isSafeInteger(result) || (result as number) < 0) {
        throw new Error(
          `${contractName}.${signature}.${stat} must be a non-negative safe integer`,
        );
      }
      return [stat, result];
    }),
  ) as unknown as GasReportFunction;
}

export function parseGasReport(raw: unknown, snapshotPrefix: string): Snapshot {
  return Object.fromEntries(
    Object.entries(GAS_REPORT_FUNCTIONS).flatMap(
      ([operation, { contract, signature }]) => {
        const report = parseGasReportFunction(raw, contract, signature);
        if (report.calls !== 50) {
          throw new Error(
            `${contract}.${signature} expected 50 calls, got ${report.calls}`,
          );
        }
        return GAS_REPORT_STATS.map((stat) => [
          `${snapshotPrefix}.${operation}.${stat}`,
          report[stat],
        ]);
      },
    ),
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

function gasReportRow(
  gasReport: Snapshot,
  proof: string,
  snapshotPrefix: string,
  operation: string,
): string {
  return `| ${proof} | ${operation} | ${value(gasReport, `${snapshotPrefix}.calls`)} | ${value(gasReport, `${snapshotPrefix}.min`)} | ${value(gasReport, `${snapshotPrefix}.mean`)} | ${value(gasReport, `${snapshotPrefix}.median`)} | ${value(gasReport, `${snapshotPrefix}.max`)} |`;
}

export function renderReadme(
  sp1: Snapshot,
  sp1GasReport: Snapshot,
  besu: Snapshot,
): string {
  const lines = [
    "<!-- This file is generated by `just solidity::test-benchmark`. Do not edit it manually. -->",
    "",
    "# Solidity Benchmarks",
    "",
    "These values are generated from deterministic Foundry tests and checked into the repository so benchmark changes are visible in GitHub diffs. They report EVM execution gas and do not include transaction intrinsic gas or calldata pricing.",
    "",
    "The benchmark profile uses Solidity 0.8.28, the Cancun EVM, IR compilation, 10,000 optimizer runs, and the fixtures committed under `test/`.",
    "",
    "Regenerate the JSON snapshots and this document from the repository root:",
    "",
    "```sh",
    "just solidity::test-benchmark",
    "```",
    "",
    "## SP1 packet function gas report",
    "",
    "These are Forge's isolated function-level gas statistics from the existing 50-packet scenarios. Top-level calls receive fresh transaction contexts while contract state advances across the scenario. `sendTransfer` is called at the top level; `ackPacket` and `recvPacket` are attributed nested calls within the proof-bearing router multicalls.",
    "",
    "| Proof fixture | Function | Calls | Min gas | Average gas | Median gas | Max gas |",
    "| --- | --- | ---: | ---: | ---: | ---: | ---: |",
    gasReportRow(
      sp1GasReport,
      "Groth16",
      "erc20.groth16.50.send_transfer",
      "sendTransfer",
    ),
    gasReportRow(
      sp1GasReport,
      "Groth16",
      "erc20.groth16.50.ack_packet",
      "ackPacket",
    ),
    gasReportRow(
      sp1GasReport,
      "Groth16",
      "erc20.groth16.50.recv_packet",
      "recvPacket",
    ),
    gasReportRow(
      sp1GasReport,
      "Plonk",
      "erc20.plonk.50.send_transfer",
      "sendTransfer",
    ),
    gasReportRow(
      sp1GasReport,
      "Plonk",
      "erc20.plonk.50.ack_packet",
      "ackPacket",
    ),
    gasReportRow(
      sp1GasReport,
      "Plonk",
      "erc20.plonk.50.recv_packet",
      "recvPacket",
    ),
    "",
    "## SP1 Tendermint end-to-end packet benchmarks",
    "",
    "These measurements capture the complete external router multicall, including SP1 proof verification, router overhead, packet handling, and application callbacks. Calldata is the complete encoded router call in bytes.",
    "",
    "### Single-packet router calls",
    "",
    "| Operation | Groth16 gas | Plonk gas | Groth16 calldata | Plonk calldata |",
    "| --- | ---: | ---: | ---: | ---: |",
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
    "These measurements use the live Besu QBFT header and proof fixture in `test/besu-bft/fixtures/qbft.json`. They measure direct light-client operations and are not directly comparable with the SP1 end-to-end packet measurements above.",
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

function serializeSnapshot(snapshot: Snapshot): string {
  return JSON.stringify(
    Object.fromEntries(
      Object.entries(snapshot)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, value]) => [key, value.toString()]),
    ),
    null,
    2,
  );
}

async function runForge(mode: Mode): Promise<void> {
  const args = [
    "forge",
    "test",
    "--match-path",
    "test/benchmarks/*.t.sol",
    "--match-test",
    "^testBenchmark_",
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

async function runGasReport(
  testName: string,
  snapshotPrefix: string,
): Promise<Snapshot> {
  const process = Bun.spawn(
    [
      "forge",
      "test",
      "--json",
      "--gas-report",
      "--isolate",
      "--gas-snapshot-emit",
      "false",
      "--match-path",
      "test/benchmarks/SP1Benchmark.t.sol",
      "--match-test",
      testName,
    ],
    {
      cwd: PROJECT_ROOT,
      stdin: "ignore",
      stdout: "pipe",
      stderr: "inherit",
    },
  );
  const [exitCode, stdout] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
  ]);
  if (exitCode !== 0) {
    throw new Error(
      `forge gas report for ${testName} failed with exit code ${exitCode}`,
    );
  }

  try {
    return parseGasReport(JSON.parse(stdout), snapshotPrefix);
  } catch (error) {
    throw new Error(`invalid Forge gas report for ${testName}`, {
      cause: error,
    });
  }
}

async function main(): Promise<void> {
  const mode = process.argv[2];
  if (mode !== "update" && mode !== "check") {
    throw new Error("usage: bun benchmarks/generate.ts <update|check>");
  }

  await runForge(mode);
  const gasReport = validateSnapshot(
    Object.fromEntries(
      Object.entries({
        ...(await runGasReport(
          "testBenchmark_ICS20Transfer_50Packets_Groth16",
          "erc20.groth16.50",
        )),
        ...(await runGasReport(
          "testBenchmark_ICS20Transfer_50Packets_Plonk",
          "erc20.plonk.50",
        )),
      }).map(([key, value]) => [key, value.toString()]),
    ),
    SP1_GAS_REPORT_KEYS,
    SP1_GAS_REPORT_FILE,
  );
  const gasReportPath = join(SNAPSHOT_DIR, SP1_GAS_REPORT_FILE);
  const expectedGasReport = serializeSnapshot(gasReport);

  if (mode === "update") {
    await writeFile(gasReportPath, expectedGasReport);
  } else {
    assertGeneratedContent(
      await readFile(gasReportPath, "utf8"),
      expectedGasReport,
      gasReportPath,
    );
  }

  const [sp1, trackedGasReport, besu] = await Promise.all([
    readSnapshot("SP1E2E.json", SP1_KEYS),
    readSnapshot(SP1_GAS_REPORT_FILE, SP1_GAS_REPORT_KEYS),
    readSnapshot("BesuQBFT.json", BESU_QBFT_KEYS),
  ]);
  const expectedReadme = renderReadme(sp1, trackedGasReport, besu);

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
