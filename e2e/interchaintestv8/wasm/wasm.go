// SPDX-License-Identifier: Apache-2.0

package wasm

import (
	"fmt"
	"io"
	"net/http"
	"os"
)

const (
	basePath                   = "e2e/interchaintestv8/wasm/"
	dummyLightClientFileName   = "cw_dummy_light_client.wasm.gz"
	wasmEthLightClientFileName = "cw_ics08_wasm_eth.wasm.gz"
)

func GetWasmDummyLightClient() (*os.File, error) {
	return os.Open(basePath + dummyLightClientFileName)
}

func GetLocalWasmEthLightClient() (*os.File, error) {
	return os.Open(basePath + wasmEthLightClientFileName)
}

func DownloadWasmLightClientRelease(release Release) (*os.File, error) {
	downloadUrl := fmt.Sprint(release.WasmDownloadURL())

	resp, err := http.Get(downloadUrl) //nolint:gosec
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	downloadFile, err := os.CreateTemp("", "eth-light-client-download")
	if err != nil {
		return nil, err
	}

	_, err = io.Copy(downloadFile, resp.Body)
	if err != nil {
		downloadFile.Close()
		os.Remove(downloadFile.Name())
		return nil, err
	}

	// Seek to the beginning of the file before returning
	_, err = downloadFile.Seek(0, io.SeekStart)
	if err != nil {
		downloadFile.Close()
		os.Remove(downloadFile.Name())
		return nil, err
	}

	return downloadFile, nil
}
