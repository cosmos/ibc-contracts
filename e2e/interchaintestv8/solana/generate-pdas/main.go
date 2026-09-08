// SPDX-License-Identifier: Apache-2.0

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	seedKindConst   = "const"
	seedKindArg     = "arg"
	seedKindAccount = "account"
)

// Configuration holds the command-line configuration
type Configuration struct {
	IDLDirectory string
	OutputFile   string
}

// IDL Types - Domain models for Anchor IDL structure
type IDL struct {
	Address      string        `json:"address"`
	Metadata     Metadata      `json:"metadata"`
	Instructions []Instruction `json:"instructions"`
}

type Metadata struct {
	Name string `json:"name"`
}

type Instruction struct {
	Name     string    `json:"name"`
	Accounts []Account `json:"accounts"`
}

type Account struct {
	Name string  `json:"name"`
	PDA  *PDADef `json:"pda,omitempty"`
}

type PDADef struct {
	Seeds []Seed `json:"seeds"`
}

type Seed struct {
	Kind  string `json:"kind"`
	Value []byte `json:"value,omitempty"`
	Path  string `json:"path,omitempty"`
}

// PDAPattern represents a unique PDA pattern to generate
type PDAPattern struct {
	Name        string
	FuncName    string
	Seeds       []Seed
	ProgramName string
	ProgramID   string
}

// Generator handles the PDA generation process
type Generator struct {
	config   *Configuration
	patterns []PDAPattern
}

// NewGenerator creates a new Generator instance
func NewGenerator(config *Configuration) *Generator {
	return &Generator{
		config: config,
	}
}

// Run executes the complete generation process
func (g *Generator) Run() error {
	// Extract PDA patterns from IDL files
	if err := g.extractPatterns(); err != nil {
		return fmt.Errorf("extracting patterns: %w", err)
	}

	// Generate Go code
	code, err := g.generateCode()
	if err != nil {
		return fmt.Errorf("generating code: %w", err)
	}

	// Write output file
	if err := os.WriteFile(g.config.OutputFile, []byte(code), 0o600); err != nil {
		return fmt.Errorf("writing output: %w", err)
	}

	fmt.Printf("Generated %d PDA helpers to %s\n", len(g.patterns), g.config.OutputFile)
	return nil
}

// extractPatterns reads all IDL files and extracts unique PDA patterns
func (g *Generator) extractPatterns() error {
	files, err := g.findIDLFiles()
	if err != nil {
		return err
	}

	patterns := make([]PDAPattern, 0)
	seenSignatures := make(map[string]bool)
	seenFuncNames := make(map[string]bool)

	for _, file := range files {
		filePatterns, err := g.extractFromFile(file)
		if err != nil {
			return fmt.Errorf("processing %s: %w", file, err)
		}

		for _, pattern := range filePatterns {
			signature := pattern.buildSignature()
			if seenSignatures[signature] {
				continue
			}
			seenSignatures[signature] = true

			pattern.FuncName = pattern.buildFuncName()
			// Skip patterns that would produce duplicate Go method names.
			// This handles IDLs with the same module name but different
			// program addresses (e.g. access_manager vs test_access_manager)
			// where program-address-derived const seeds differ but the
			// generated method name is identical. The method takes programID
			// as a runtime parameter so a single helper suffices.
			if seenFuncNames[pattern.FuncName] {
				continue
			}
			seenFuncNames[pattern.FuncName] = true

			patterns = append(patterns, pattern)
		}
	}

	// Sort patterns by function name for consistent output
	sort.Slice(patterns, func(i, j int) bool {
		return patterns[i].FuncName < patterns[j].FuncName
	})

	g.patterns = patterns
	return nil
}

// findIDLFiles discovers all JSON files in the IDL directory
func (g *Generator) findIDLFiles() ([]string, error) {
	entries, err := os.ReadDir(g.config.IDLDirectory)
	if err != nil {
		return nil, fmt.Errorf("reading IDL directory: %w", err)
	}

	var files []string
	for _, entry := range entries {
		if !entry.IsDir() && strings.HasSuffix(entry.Name(), ".json") {
			files = append(files, filepath.Join(g.config.IDLDirectory, entry.Name()))
		}
	}

	if len(files) == 0 {
		return nil, fmt.Errorf("no IDL JSON files found in %s", g.config.IDLDirectory)
	}

	sort.Strings(files)
	return files, nil
}

// extractFromFile extracts PDA patterns from a single IDL file
func (g *Generator) extractFromFile(path string) ([]PDAPattern, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("reading file: %w", err)
	}

	var idl IDL
	if err := json.Unmarshal(data, &idl); err != nil {
		return nil, fmt.Errorf("parsing JSON: %w", err)
	}

	programName := toPascalCase(idl.Metadata.Name)
	var patterns []PDAPattern

	for _, instruction := range idl.Instructions {
		for _, account := range instruction.Accounts {
			if account.PDA != nil {
				patterns = append(patterns, PDAPattern{
					Name:        account.Name,
					Seeds:       account.PDA.Seeds,
					ProgramName: programName,
					ProgramID:   idl.Address,
				})
			}
		}
	}

	return patterns, nil
}

// buildSignature creates a unique signature for deduplication
func (p *PDAPattern) buildSignature() string {
	var parts []string
	parts = append(parts, p.ProgramName)

	for _, seed := range p.Seeds {
		if seed.Kind == seedKindConst {
			// Use hex encoding for binary data to ensure valid signature
			if isPrintableASCII(seed.Value) {
				parts = append(parts, string(seed.Value))
			} else {
				parts = append(parts, fmt.Sprintf("0x%x", seed.Value))
			}
		} else {
			parts = append(parts, seed.Kind)
		}
	}

	return strings.Join(parts, "|")
}

// buildFuncName generates the function name for this PDA pattern
func (p *PDAPattern) buildFuncName() string {
	builder := &funcNameBuilder{
		pattern: p,
	}
	return builder.build()
}

// funcNameBuilder helps construct function names
type funcNameBuilder struct {
	pattern *PDAPattern
}

func (b *funcNameBuilder) build() string {
	nameParts := b.extractNameParts()
	baseName := b.pattern.ProgramName + strings.Join(nameParts, "")

	// Add suffix based on dynamic seed types to differentiate patterns
	hasArg := b.hasArgSeed()
	hasAccount := b.hasAccountSeed()

	switch {
	case hasArg && hasAccount:
		baseName += "WithArgAndAccountSeed"
	case hasArg:
		baseName += "WithArgSeed"
	case hasAccount:
		baseName += "WithAccountSeed"
	}

	return baseName + "PDA"
}

func (b *funcNameBuilder) extractNameParts() []string {
	var parts []string

	for _, seed := range b.pattern.Seeds {
		if seed.Kind == seedKindConst {
			// Only use const seeds that are printable ASCII for function names
			if isPrintableASCII(seed.Value) {
				parts = append(parts, toPascalCase(string(seed.Value)))
			}
		}
	}

	// Use account name if no printable const seeds found
	if len(parts) == 0 {
		parts = append(parts, toPascalCase(b.pattern.Name))
	}

	return parts
}

func (b *funcNameBuilder) hasAccountSeed() bool {
	for _, seed := range b.pattern.Seeds {
		if seed.Kind == seedKindAccount {
			return true
		}
	}
	return false
}

func (b *funcNameBuilder) hasArgSeed() bool {
	for _, seed := range b.pattern.Seeds {
		if seed.Kind == seedKindArg {
			return true
		}
	}
	return false
}

// CodeGenerator handles code generation
type CodeGenerator struct {
	patterns []PDAPattern
}

// generateCode creates the Go source code
func (g *Generator) generateCode() (string, error) {
	cg := &CodeGenerator{patterns: g.patterns}
	return cg.generate()
}

func (cg *CodeGenerator) generate() (string, error) {
	var b strings.Builder

	// Write file header
	b.WriteString(cg.generateHeader())

	// Group patterns by program
	programPatterns := cg.groupByProgram()

	// Get sorted program names for consistent output
	var programNames []string
	for name := range programPatterns {
		programNames = append(programNames, name)
	}
	sort.Strings(programNames)

	// Generate type definitions and singleton instances
	b.WriteString(cg.generateTypes(programNames))
	b.WriteString("\n")

	// Generate methods for each program in sorted order
	for _, programName := range programNames {
		patterns := programPatterns[programName]
		for _, pattern := range patterns {
			b.WriteString(cg.generateMethod(programName, pattern))
			b.WriteString("\n")
		}
	}

	return b.String(), nil
}

func (cg *CodeGenerator) groupByProgram() map[string][]PDAPattern {
	programPatterns := make(map[string][]PDAPattern)
	for _, pattern := range cg.patterns {
		programPatterns[pattern.ProgramName] = append(programPatterns[pattern.ProgramName], pattern)
	}
	return programPatterns
}

func (cg *CodeGenerator) generateTypes(programNames []string) string {
	var b strings.Builder

	// Generate type definitions
	for _, name := range programNames {
		typeName := strings.ToLower(name[:1]) + name[1:] + "PDAs"
		fmt.Fprintf(&b, "type %s struct{}\n", typeName)
	}
	b.WriteString("\n")

	// Generate singleton instances
	b.WriteString("var (\n")
	for _, name := range programNames {
		typeName := strings.ToLower(name[:1]) + name[1:] + "PDAs"
		fmt.Fprintf(&b, "\t%s = %s{}\n", name, typeName)
	}
	b.WriteString(")\n")

	return b.String()
}

func (cg *CodeGenerator) generateHeader() string {
	return `// Code generated by solana/generate-pdas. DO NOT EDIT.
//
// This file is automatically generated from Anchor IDL files.
// Run 'just solana::generate-pda' to regenerate.
//
// DO NOT EDIT THIS FILE MANUALLY.

package solana

import (
	"fmt"

	solanago "github.com/gagliardetto/solana-go"
)

`
}

func (cg *CodeGenerator) generateMethod(programName string, p PDAPattern) string {
	fg := &functionGenerator{
		pattern:     p,
		programName: programName,
	}
	return fg.generate()
}

// functionGenerator generates a single PDA method
type functionGenerator struct {
	pattern     PDAPattern
	programName string
}

func (fg *functionGenerator) generate() string {
	var b strings.Builder

	// Function signature
	b.WriteString(fg.generateSignature())
	b.WriteString(" {\n")

	// Seeds array
	b.WriteString(fg.generateSeedsArray())

	// PDA derivation
	b.WriteString(fg.generatePDADerivation())

	// Error handling
	b.WriteString(fg.generateErrorHandling())

	// Return statement
	b.WriteString("\treturn pda, bump\n")
	b.WriteString("}\n")

	return b.String()
}

func (fg *functionGenerator) generateSignature() string {
	params := fg.extractParameters()
	receiverType := strings.ToLower(fg.programName[:1]) + fg.programName[1:] + "PDAs"

	// Strip program name prefix from method name
	methodName := strings.TrimPrefix(fg.pattern.FuncName, fg.programName)

	return fmt.Sprintf("func (%s) %s(%s) (solanago.PublicKey, uint8)",
		receiverType, methodName, params)
}

func (fg *functionGenerator) extractParameters() string {
	params := []string{"programID solanago.PublicKey"}
	seen := make(map[string]bool)

	for _, seed := range fg.pattern.Seeds {
		if seed.Kind == seedKindArg || seed.Kind == seedKindAccount {
			paramName := extractParamName(seed.Path)
			paramKey := fmt.Sprintf("%s_%s", seed.Kind, paramName)

			if !seen[paramKey] {
				params = append(params, fmt.Sprintf("%s []byte", paramName))
				seen[paramKey] = true
			}
		}
	}

	return strings.Join(params, ", ")
}

func (fg *functionGenerator) generateSeedsArray() string {
	seedsCode := fg.buildSeedsCode()
	return fmt.Sprintf("\tpda, bump, err := solanago.FindProgramAddress(\n\t\t[][]byte{%s},\n\t\tprogramID,\n\t)\n",
		strings.Join(seedsCode, ", "))
}

func (fg *functionGenerator) buildSeedsCode() []string {
	var seeds []string

	for _, seed := range fg.pattern.Seeds {
		switch seed.Kind {
		case seedKindConst:
			seeds = append(seeds, formatBytesLiteral(seed.Value))
		case seedKindArg, seedKindAccount:
			seeds = append(seeds, extractParamName(seed.Path))
		}
	}

	return seeds
}

func (fg *functionGenerator) generatePDADerivation() string {
	return ""
}

func (fg *functionGenerator) generateErrorHandling() string {
	methodName := strings.TrimPrefix(fg.pattern.FuncName, fg.programName)
	return fmt.Sprintf("\tif err != nil {\n\t\tpanic(fmt.Sprintf(\"failed to derive %s.%s PDA: %%v\", err))\n\t}\n",
		fg.programName, methodName)
}

// String manipulation utilities

// isPrintableASCII checks if all bytes in the slice are printable ASCII characters
func isPrintableASCII(data []byte) bool {
	for _, b := range data {
		// Only allow printable ASCII (space to tilde: 0x20-0x7E)
		if b < 0x20 || b > 0x7E {
			return false
		}
	}
	return len(data) > 0
}

// formatBytesLiteral formats a byte slice as Go code
func formatBytesLiteral(data []byte) string {
	if isPrintableASCII(data) {
		return fmt.Sprintf("[]byte(\"%s\")", string(data))
	}
	// Format as byte slice literal for binary data
	parts := make([]string, len(data))
	for i, b := range data {
		parts[i] = fmt.Sprintf("0x%02x", b)
	}
	return fmt.Sprintf("[]byte{%s}", strings.Join(parts, ", "))
}

func toPascalCase(s string) string {
	s = strings.ReplaceAll(s, "-", "_")
	parts := strings.Split(s, "_")

	for i, part := range parts {
		if len(part) > 0 {
			parts[i] = strings.ToUpper(part[:1]) + strings.ToLower(part[1:])
		}
	}

	return strings.Join(parts, "")
}

func toCamelCase(s string) string {
	parts := strings.Split(s, "_")

	for i := 1; i < len(parts); i++ {
		if len(parts[i]) > 0 {
			parts[i] = strings.ToUpper(parts[i][:1]) + parts[i][1:]
		}
	}

	return strings.Join(parts, "")
}

func extractParamName(path string) string {
	parts := strings.Split(path, ".")
	name := parts[len(parts)-1]
	return toCamelCase(name)
}

// parseConfiguration parses command-line arguments
func parseConfiguration() (*Configuration, error) {
	var config Configuration

	flag.StringVar(&config.IDLDirectory, "idl-dir", "", "Directory containing IDL JSON files")
	flag.StringVar(&config.OutputFile, "output", "", "Output Go file")
	flag.Parse()

	if config.IDLDirectory == "" {
		return nil, fmt.Errorf("--idl-dir is required")
	}

	if config.OutputFile == "" {
		return nil, fmt.Errorf("--output is required")
	}

	// Validate IDL directory exists
	info, err := os.Stat(config.IDLDirectory)
	if err != nil {
		return nil, fmt.Errorf("IDL directory error: %w", err)
	}
	if !info.IsDir() {
		return nil, fmt.Errorf("IDL path is not a directory: %s", config.IDLDirectory)
	}

	return &config, nil
}

func main() {
	config, err := parseConfiguration()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	generator := NewGenerator(config)
	if err := generator.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Generation failed: %v\n", err)
		os.Exit(1)
	}
}
