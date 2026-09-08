// SPDX-License-Identifier: Apache-2.0

package main

import (
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/token"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetGitHubActionMatrixForTests(t *testing.T) {
	e2eDir := filepath.Clean(filepath.Join("..", "..", "e2e", "interchaintestv8"))

	matrix, err := getGitHubActionMatrixForTests(e2eDir, "", nil)
	require.NoError(t, err)

	assert.NotEmpty(t, matrix.Include, "Should discover tests")

	found := make(map[string]bool)
	for _, test := range matrix.Include {
		testKey := test.EntryPoint + "/" + test.Test
		found[testKey] = true
	}

	expectedTests := []string{
		"TestWithIbcEurekaTestSuite/Test_Deploy",
		"TestWithProofAPITestSuite/Test_2_ConcurrentRecvPacketToEth",
		"TestWithSP1ICS07TendermintTestSuite/Test_UpdateClient",
	}

	for _, expected := range expectedTests {
		assert.True(t, found[expected], "Should find test: %s", expected)
	}
}

func TestFilterBySuiteEntrypoint(t *testing.T) {
	e2eDir := filepath.Clean(filepath.Join("..", "..", "e2e", "interchaintestv8"))

	suiteName := "TestWithSP1ICS07TendermintTestSuite"
	matrix, err := getGitHubActionMatrixForTests(e2eDir, suiteName, nil)
	require.NoError(t, err)

	assert.True(t, len(matrix.Include) >= 1, "Should have at least 1 test when filtering by suite")

	for _, test := range matrix.Include {
		assert.Equal(t, suiteName, test.EntryPoint, "All tests should be from the selected suite")
	}
}

func TestFilterByExclusions(t *testing.T) {
	e2eDir := filepath.Clean(filepath.Join("..", "..", "e2e", "interchaintestv8"))

	excludedSuites := []string{"TestWithProofAPITestSuite"}
	matrix, err := getGitHubActionMatrixForTests(e2eDir, "", excludedSuites)
	require.NoError(t, err)

	for _, test := range matrix.Include {
		assert.NotEqual(t, "TestWithProofAPITestSuite", test.EntryPoint, "Should not contain excluded suite")
	}
}

func TestJSONOutput(t *testing.T) {
	testPairs := []testSuitePair{
		{Test: "Test_Deploy", EntryPoint: "TestWithIbcEurekaTestSuite"},
		{Test: "Test_UpdateClient", EntryPoint: "TestWithSP1ICS07TendermintTestSuite"},
	}

	matrix := actionTestMatrix{Include: testPairs}
	output, err := json.Marshal(matrix)
	require.NoError(t, err)

	var result actionTestMatrix
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	assert.Equal(t, testPairs, result.Include)
}

func TestIsSuiteEntrypoint(t *testing.T) {
	tests := []struct {
		name     string
		code     string
		expected bool
	}{
		{
			name: "valid suite entrypoint",
			code: `package main
import "testing"
func TestSomething(t *testing.T) {
	suite.Run(t, new(TestSuite))
}`,
			expected: true,
		},
		{
			name: "test function without suite.Run call",
			code: `package main
import "testing"
func TestSomething(t *testing.T) {
	t.Log("test")
}`,
			expected: false,
		},
		{
			name: "non-test function",
			code: `package main
import "testing"
func Something(t *testing.T) {
	suite.Run(t, new(TestSuite))
}`,
			expected: false,
		},
		{
			name: "test function with wrong parameter type",
			code: `package main
func TestSomething(t string) {
	suite.Run(t, new(TestSuite))
}`,
			expected: false,
		},
		{
			name: "test function with no parameters",
			code: `package main
func TestSomething() {
	suite.Run(nil, new(TestSuite))
}`,
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fset := token.NewFileSet()
			file, err := parser.ParseFile(fset, "", tt.code, 0)
			require.NoError(t, err)

			var funcDecl *ast.FuncDecl
			for _, decl := range file.Decls {
				if f, ok := decl.(*ast.FuncDecl); ok && (f.Name.Name == "TestSomething" || f.Name.Name == "Something") {
					funcDecl = f
					break
				}
			}
			require.NotNil(t, funcDecl, "function not found")

			result := isSuiteEntrypoint(funcDecl)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestIsSuiteTest(t *testing.T) {
	tests := []struct {
		name     string
		code     string
		expected bool
	}{
		{
			name: "valid test method on test suite",
			code: `package main
type MyTestSuite struct{}
func (s *MyTestSuite) TestSomething() {}`,
			expected: true,
		},
		{
			name: "valid test method on suite",
			code: `package main
type MySuite struct{}
func (s *MySuite) TestSomething() {}`,
			expected: true,
		},
		{
			name: "non-test method on test suite",
			code: `package main
type MyTestSuite struct{}
func (s *MyTestSuite) Something() {}`,
			expected: false,
		},
		{
			name: "test function without receiver",
			code: `package main
func TestSomething() {}`,
			expected: false,
		},
		{
			name: "test method on non-suite type",
			code: `package main
type MyStruct struct{}
func (s *MyStruct) TestSomething() {}`,
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fset := token.NewFileSet()
			file, err := parser.ParseFile(fset, "", tt.code, 0)
			require.NoError(t, err)

			var funcDecl *ast.FuncDecl
			for _, decl := range file.Decls {
				if f, ok := decl.(*ast.FuncDecl); ok && (f.Name.Name == "TestSomething" || f.Name.Name == "Something") {
					funcDecl = f
					break
				}
			}
			require.NotNil(t, funcDecl, "function not found")

			result := isSuiteTest(funcDecl)
			assert.Equal(t, tt.expected, result)
		})
	}
}
