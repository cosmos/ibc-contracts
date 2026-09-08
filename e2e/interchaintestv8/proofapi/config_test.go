// SPDX-License-Identifier: Apache-2.0

package proofapi

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"
)

func TestGenerateConfigFileEscapesStringsAndWritesEmptyModules(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")
	config := Config{Server: ServerConfig{Address: "quoted \"address\""}}

	if err := config.GenerateConfigFile(path); err != nil {
		t.Fatal(err)
	}
	contents, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Contains(contents, []byte(`"modules": []`)) {
		t.Errorf("config contains non-empty or null modules: %s", contents)
	}
	if !bytes.Contains(contents, []byte(`"address": "quoted \"address\""`)) {
		t.Errorf("config does not escape address: %s", contents)
	}
}
