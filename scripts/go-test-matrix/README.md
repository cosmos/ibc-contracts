<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# go-test-matrix

This tool scans Go test files under `e2e/interchaintestv8` to discover testify suites and their test methods, then outputs a JSON matrix suitable for GitHub Actions.

## Usage

```bash
# Generate matrix for all tests
go run main.go

# Filter to a specific test suite (only tests from that suite)
TEST_ENTRYPOINT=TestWithIbcEurekaTestSuite go run main.go

# Exclude specific test suites
TEST_EXCLUSIONS=TestWithCosmosProofAPITestSuite,TestWithMultichainTestSuite go run main.go
```

## Environment Variables

- `TEST_ENTRYPOINT`: Return only tests from the given suite entrypoint (e.g. `TestWithIbcEurekaTestSuite`)
- `TEST_EXCLUSIONS`: Comma-separated list of suite entrypoints to exclude

## Output

The tool outputs a JSON object compatible with GitHub Actions matrix strategy:

```json
{
  "include": [
    {"test": "Test_Deploy", "entrypoint": "TestWithIbcEurekaTestSuite"},
    {"test": "Test_2_ConcurrentRecvPacketToEth", "entrypoint": "TestWithProofAPITestSuite"}
  ]
}
```

## Test Discovery

The tool discovers tests by:

1. Parsing Go files using the Go AST parser
2. Finding top-level `Test*` functions that invoke `suite.Run(...)` (testify entrypoints)
3. Finding suite test methods that match `func (s *SuiteName) Test*` where the receiver type ends with `Suite` or `TestSuite`
4. Emitting pairs of `{ test: <method name>, entrypoint: <top-level suite function> }`
