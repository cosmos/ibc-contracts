// SPDX-License-Identifier: Apache-2.0

import { describe, expect, test } from "bun:test";

import {
  assertGeneratedContent,
  averagePerPacket,
  formatInteger,
  parseGasReport,
  SP1_GAS_REPORT_KEYS,
  validateSnapshot,
} from "./generate";

describe("benchmark generator", () => {
  test("validates and parses Foundry snapshot values", () => {
    expect(
      validateSnapshot(
        { gas: "1234", calldata: "56" },
        ["calldata", "gas"],
        "fixture",
      ),
    ).toEqual({
      gas: 1234,
      calldata: 56,
    });
  });

  test("rejects missing and unexpected snapshot keys", () => {
    expect(() =>
      validateSnapshot(
        { gas: "1234", extra: "1" },
        ["gas", "calldata"],
        "fixture",
      ),
    ).toThrow("missing: calldata; unexpected: extra");
  });

  test("rejects non-integer snapshot values", () => {
    expect(() => validateSnapshot({ gas: 1234 }, ["gas"], "fixture")).toThrow(
      "must be an unsigned integer encoded as a string",
    );
  });

  test("formats integers and derives per-packet averages deterministically", () => {
    expect(formatInteger(1234567)).toBe("1,234,567");
    expect(averagePerPacket(100, 6)).toBe(16);
  });

  test("extracts isolated function statistics from Forge's JSON gas report", () => {
    const stats = { calls: 50, min: 10, mean: 20, median: 15, max: 100 };
    const report = parseGasReport(
      [
        {
          contract: "contracts/ICS20Transfer.sol:ICS20Transfer",
          functions: {
            "sendTransfer((address,uint256,string,string,string,uint64,string))":
              stats,
          },
        },
        {
          contract: "contracts/ICS26Router.sol:ICS26Router",
          functions: {
            "ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)))":
              stats,
            "recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)))":
              stats,
          },
        },
      ],
      "erc20.plonk.50",
    );

    expect(report["erc20.plonk.50.send_transfer.mean"]).toBe(20);
    expect(report["erc20.plonk.50.recv_packet.max"]).toBe(100);
    expect(SP1_GAS_REPORT_KEYS).toHaveLength(30);
  });

  test("detects stale generated content", () => {
    expect(() => assertGeneratedContent("old", "new", "README.md")).toThrow(
      "README.md is stale; run `just solidity::test-benchmark`",
    );
    expect(() =>
      assertGeneratedContent("same", "same", "README.md"),
    ).not.toThrow();
  });
});
