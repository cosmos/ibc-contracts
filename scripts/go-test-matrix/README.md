# go-test-matrix

This tool scans the Go test files under the directory passed with `-dir` (in CI, `e2e/interchaintestv8`) to discover testify suites and their test methods, then outputs a JSON matrix suitable for GitHub Actions.

## Usage

From the repo root (the `-dir` flag is required):

```bash
# Generate matrix for all tests
go run scripts/go-test-matrix/main.go -dir e2e/interchaintestv8

# Filter to a specific test suite (only tests from that suite)
TEST_ENTRYPOINT=TestWithIbcEurekaTestSuite go run scripts/go-test-matrix/main.go -dir e2e/interchaintestv8

# Exclude specific test suites, or single tests as `<entrypoint>/<test>`
TEST_EXCLUSIONS=TestWithCosmosProofAPITestSuite,TestWithIbcEurekaTestSuite/Test_Deploy go run scripts/go-test-matrix/main.go -dir e2e/interchaintestv8
```

## Environment Variables

- `TEST_ENTRYPOINT`: Return only tests from the given suite entrypoint (e.g. `TestWithIbcEurekaTestSuite`)
- `TEST_EXCLUSIONS`: Comma-separated list of suite entrypoints, or `<entrypoint>/<test>` names, to exclude

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

1. Parsing `_test.go` files using the Go AST parser
2. Finding top-level `Test*(t *testing.T)` functions that invoke `suite.Run(...)` (testify entrypoints)
3. Finding suite test methods that match `func (s *SuiteName) Test*` where the receiver type ends with `Suite` or `TestSuite`
4. Emitting pairs of `{ test: <method name>, entrypoint: <top-level suite function> }`
