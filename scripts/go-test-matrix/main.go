// SPDX-License-Identifier: Apache-2.0

package main

import (
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io/fs"
	"os"
	"path/filepath"
	"slices"
	"sort"
	"strings"
)

const (
	testNamePrefix     = "Test"
	testFileNameSuffix = "_test.go"

	// testEntryPointEnv is an optional env variable that can be used to only return tests for a specific suite
	testEntryPointEnv = "TEST_ENTRYPOINT"

	// testExclusionsEnv is an optional env variable that can be used to exclude tests, or entire suites, from the output
	testExclusionsEnv = "TEST_EXCLUSIONS"
)

type actionTestMatrix struct {
	Include []testSuitePair `json:"include"`
}

type testSuitePair struct {
	Test       string `json:"test"`
	EntryPoint string `json:"entrypoint"`
}

var (
	ErrNoSuiteEntrypoint       = errors.New("no suite entrypoint found")
	ErrMultipleSuiteEntrypoint = errors.New("multiple suite entrypoints found")
)

func main() {
	var testDir string
	flag.StringVar(&testDir, "dir", "", "Path to the test directory (required)")
	flag.Parse()

	if testDir == "" {
		fmt.Fprintln(os.Stderr, "error: -dir flag is required")
		flag.Usage()
		os.Exit(1)
	}

	suite := os.Getenv(testEntryPointEnv)
	var excludedItems []string
	if exclusions, ok := os.LookupEnv(testExclusionsEnv); ok {
		excludedItems = strings.Split(exclusions, ",")
	}

	// Verify the test directory exists
	if _, err := os.Stat(testDir); err != nil {
		fmt.Fprintf(os.Stderr, "error: test directory '%s' does not exist: %v\n", testDir, err)
		os.Exit(1)
	}

	matrix, err := getGitHubActionMatrixForTests(testDir, suite, excludedItems)
	if err != nil {
		fmt.Fprintln(os.Stderr, "error generating GitHub Action JSON:", err)
		os.Exit(1)
	}

	if err := json.NewEncoder(os.Stdout).Encode(matrix); err != nil {
		fmt.Fprintln(os.Stderr, "error writing JSON:", err)
		os.Exit(1)
	}
}

func getGitHubActionMatrixForTests(e2eRootDirectory, suite string, excludedItems []string) (actionTestMatrix, error) {
	testSuiteMapping := map[string][]string{}

	fileSet := token.NewFileSet()
	err := filepath.WalkDir(e2eRootDirectory, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return fmt.Errorf("walk e2e: %w", err)
		}

		if d.IsDir() || !strings.HasSuffix(path, testFileNameSuffix) {
			return nil
		}

		astFile, err := parser.ParseFile(fileSet, path, nil, 0)
		if err != nil {
			return fmt.Errorf("parse file: %w", err)
		}

		suiteName, suiteTestCases, err := extractSuiteAndTestNames(astFile)
		if err != nil {
			// Ignore files without suite entrypoints (regular test files)
			if errors.Is(err, ErrNoSuiteEntrypoint) {
				return nil
			}
			// Propagate all other errors (like multiple suite entrypoints)
			return fmt.Errorf("in file %s: %w", path, err)
		}

		if slices.Contains(excludedItems, suiteName) {
			return nil
		}

		if suite == "" || suiteName == suite {
			testSuiteMapping[suiteName] = suiteTestCases
		}

		return nil
	})
	if err != nil {
		return actionTestMatrix{}, err
	}

	gh := actionTestMatrix{
		Include: []testSuitePair{},
	}

	for testSuiteName, testCases := range testSuiteMapping {
		for _, testCaseName := range testCases {
			// Check if this specific test is excluded
			fullTestName := fmt.Sprintf("%s/%s", testSuiteName, testCaseName)
			if slices.Contains(excludedItems, fullTestName) {
				continue
			}

			gh.Include = append(gh.Include, testSuitePair{
				Test:       testCaseName,
				EntryPoint: testSuiteName,
			})
		}
	}

	if len(gh.Include) == 0 {
		return actionTestMatrix{}, errors.New("no test cases found")
	}

	sort.Slice(gh.Include, func(i, j int) bool {
		if gh.Include[i].EntryPoint != gh.Include[j].EntryPoint {
			return gh.Include[i].EntryPoint < gh.Include[j].EntryPoint
		}
		if gh.Include[i].Test != gh.Include[j].Test {
			return gh.Include[i].Test < gh.Include[j].Test
		}
		return false
	})

	return gh, nil
}

// extractSuiteAndTestNames extracts the suite name and test names from a Go file by parsing the AST.
func extractSuiteAndTestNames(file *ast.File) (string, []string, error) {
	suiteName := ""
	testNames := []string{}

	for _, declaration := range file.Decls {
		fn, ok := declaration.(*ast.FuncDecl)
		if !ok {
			continue
		}

		fnName := fn.Name.Name

		switch {
		case isSuiteEntrypoint(fn):
			if suiteName != "" {
				return "", nil, fmt.Errorf("%w: %s and %s", ErrMultipleSuiteEntrypoint, suiteName, fnName)
			}
			suiteName = fnName
		case isSuiteTest(fn):
			testNames = append(testNames, fnName)
		}
	}

	if suiteName == "" {
		return "", nil, fmt.Errorf("%w in file %s", ErrNoSuiteEntrypoint, file.Name.Name)
	}

	return suiteName, testNames, nil
}

func isSuiteEntrypoint(f *ast.FuncDecl) bool {
	if !isTestFunction(f) {
		return false
	}

	return callsTestifySuiteRun(f)
}

func isTestFunction(fn *ast.FuncDecl) bool {
	if !strings.HasPrefix(fn.Name.Name, testNamePrefix) {
		return false
	}
	if len(fn.Type.Params.List) != 1 {
		return false
	}
	paramField := fn.Type.Params.List[0]
	pointerType, ok := paramField.Type.(*ast.StarExpr)
	if !ok {
		return false
	}
	selectorType, ok := pointerType.X.(*ast.SelectorExpr)
	if !ok {
		return false
	}
	pkgIdent, ok := selectorType.X.(*ast.Ident)
	if !ok {
		return false
	}
	if pkgIdent.Name != "testing" || selectorType.Sel.Name != "T" {
		return false
	}

	return true
}

func callsTestifySuiteRun(fn *ast.FuncDecl) bool {
	if fn.Body == nil {
		return false
	}

	for _, statement := range fn.Body.List {
		exprStatement, ok := statement.(*ast.ExprStmt)
		if !ok {
			continue
		}

		callExpression, ok := exprStatement.X.(*ast.CallExpr)
		if !ok {
			continue
		}

		selectorExpression, ok := callExpression.Fun.(*ast.SelectorExpr)
		if !ok {
			continue
		}

		receiverIdent, ok := selectorExpression.X.(*ast.Ident)
		if !ok {
			continue
		}

		if receiverIdent.Name == "suite" && selectorExpression.Sel.Name == "Run" {
			return true
		}
	}

	return false
}

func isSuiteTest(fn *ast.FuncDecl) bool {
	if !strings.HasPrefix(fn.Name.Name, testNamePrefix) {
		return false
	}
	if fn.Recv == nil || len(fn.Recv.List) != 1 {
		return false
	}

	receiverField := fn.Recv.List[0]
	pointerType, ok := receiverField.Type.(*ast.StarExpr)
	if !ok {
		return false
	}
	receiverIdent, ok := pointerType.X.(*ast.Ident)
	if !ok {
		return false
	}

	return strings.HasSuffix(receiverIdent.Name, "TestSuite") || strings.HasSuffix(receiverIdent.Name, "Suite")
}
