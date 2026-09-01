// SPDX-License-Identifier: Apache-2.0

import { describe, expect, test } from "bun:test";

import {
  assertGeneratedContent,
  averagePerPacket,
  formatInteger,
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

  test("detects stale generated content", () => {
    expect(() => assertGeneratedContent("old", "new", "README.md")).toThrow(
      "README.md is stale; run `just solidity::test-benchmark`",
    );
    expect(() =>
      assertGeneratedContent("same", "same", "README.md"),
    ).not.toThrow();
  });
});
