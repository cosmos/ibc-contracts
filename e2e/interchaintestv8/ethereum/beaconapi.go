// SPDX-License-Identifier: Apache-2.0

package ethereum

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	eth2client "github.com/attestantio/go-eth2-client"
	"github.com/attestantio/go-eth2-client/api"
	apiv1 "github.com/attestantio/go-eth2-client/api/v1"
	ethttp "github.com/attestantio/go-eth2-client/http"
	"github.com/attestantio/go-eth2-client/spec/phase0"
	"github.com/rs/zerolog"
)

type BeaconAPIClient struct {
	ctx    context.Context
	cancel context.CancelFunc

	client eth2client.Service
	url    string

	Retries   int
	RetryWait time.Duration
}

func (b BeaconAPIClient) GetBeaconAPIURL() string {
	return b.url
}

func (s Spec) Period() uint64 {
	return s.EpochsPerSyncCommitteePeriod * s.SlotsPerEpoch
}

func (b BeaconAPIClient) Close() {
	b.cancel()
}

func NewBeaconAPIClient(ctx context.Context, beaconAPIAddress string) (BeaconAPIClient, error) {
	ctx, cancel := context.WithCancel(ctx)
	client, err := ethttp.New(ctx,
		// WithAddress supplies the address of the beacon node, as a URL.
		ethttp.WithAddress(beaconAPIAddress),
		// LogLevel supplies the level of logging to carry out.
		ethttp.WithLogLevel(zerolog.WarnLevel),
	)
	if err != nil {
		cancel()
		return BeaconAPIClient{}, err
	}

	return BeaconAPIClient{
		ctx:       ctx,
		cancel:    cancel,
		client:    client,
		url:       beaconAPIAddress,
		Retries:   60,
		RetryWait: 10 * time.Second,
	}, nil
}

func retry[T any](retries int, waitTime time.Duration, fn func() (T, error)) (T, error) {
	var err error
	var result T
	for range retries {
		result, err = fn()
		if err == nil {
			return result, nil
		}

		fmt.Printf("Retrying for %T: %s in %f seconds\n", result, err, waitTime.Seconds())
		time.Sleep(waitTime)
	}
	return result, err
}

func (b BeaconAPIClient) GetHeader(blockID string) (*apiv1.BeaconBlockHeader, error) {
	return retry(b.Retries, b.RetryWait, func() (*apiv1.BeaconBlockHeader, error) {
		headerResponse, err := b.client.(eth2client.BeaconBlockHeadersProvider).BeaconBlockHeader(b.ctx, &api.BeaconBlockHeaderOpts{
			Block: blockID,
		})
		if err != nil {
			return nil, err
		}

		return headerResponse.Data, nil
	})
}

func (b BeaconAPIClient) GetBootstrap(finalizedRoot phase0.Root) (Bootstrap, error) {
	return retry(b.Retries, b.RetryWait, func() (Bootstrap, error) {
		finalizedRootStr := finalizedRoot.String()
		url := fmt.Sprintf("%s/eth/v1/beacon/light_client/bootstrap/%s", b.url, finalizedRootStr)
		req, err := http.NewRequest("GET", url, nil)
		if err != nil {
			return Bootstrap{}, err
		}
		req.Header.Set("Accept", "application/json")
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			return Bootstrap{}, err
		}
		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return Bootstrap{}, err
		}

		if resp.StatusCode != 200 {
			return Bootstrap{}, fmt.Errorf("get bootstrap (%s) failed with status code: %d, body: %s", url, resp.StatusCode, body)
		}

		var bootstrap Bootstrap
		if err := json.Unmarshal(body, &bootstrap); err != nil {
			return Bootstrap{}, err
		}

		return bootstrap, nil
	})
}

func (b BeaconAPIClient) GetSpec() (Spec, error) {
	return retry(b.Retries, b.RetryWait, func() (Spec, error) {
		specResponse, err := b.client.(eth2client.SpecProvider).Spec(b.ctx, &api.SpecOpts{})
		if err != nil {
			return Spec{}, err
		}

		specJsonBz, err := json.Marshal(specResponse.Data)
		if err != nil {
			return Spec{}, err
		}

		var spec Spec
		if err := json.Unmarshal(specJsonBz, &spec); err != nil {
			return Spec{}, err
		}

		return spec, nil
	})
}

func (b BeaconAPIClient) GetFinalityUpdate() (FinalityUpdateJSONResponse, error) {
	return retry(b.Retries, b.RetryWait, func() (FinalityUpdateJSONResponse, error) {
		url := fmt.Sprintf("%s/eth/v1/beacon/light_client/finality_update", b.url)
		req, err := http.NewRequest("GET", url, nil)
		if err != nil {
			return FinalityUpdateJSONResponse{}, err
		}
		req.Header.Set("Accept", "application/json")

		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			return FinalityUpdateJSONResponse{}, err
		}
		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return FinalityUpdateJSONResponse{}, err
		}

		var finalityUpdate FinalityUpdateJSONResponse
		if err := json.Unmarshal(body, &finalityUpdate); err != nil {
			return FinalityUpdateJSONResponse{}, err
		}

		return finalityUpdate, nil
	})
}

func (b BeaconAPIClient) GetBeaconBlocks(blockID string) (BeaconBlocksResponseJSON, error) {
	return retry(b.Retries, b.RetryWait, func() (BeaconBlocksResponseJSON, error) {
		url := fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.url, blockID)
		req, err := http.NewRequest("GET", url, nil)
		if err != nil {
			return BeaconBlocksResponseJSON{}, err
		}

		req.Header.Set("Accept", "application/json")
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			return BeaconBlocksResponseJSON{}, err
		}

		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return BeaconBlocksResponseJSON{}, err
		}

		if resp.StatusCode != 200 {
			return BeaconBlocksResponseJSON{}, fmt.Errorf("get execution height (%s) failed with status code: %d, body: %s", url, resp.StatusCode, body)
		}

		var beaconBlocksResponse BeaconBlocksResponseJSON
		if err := json.Unmarshal(body, &beaconBlocksResponse); err != nil {
			return BeaconBlocksResponseJSON{}, err
		}

		return beaconBlocksResponse, nil
	})
}

func (b BeaconAPIClient) GetFinalizedBlocks() (BeaconBlocksResponseJSON, error) {
	return retry(b.Retries, b.RetryWait, func() (BeaconBlocksResponseJSON, error) {
		resp, err := b.GetBeaconBlocks("finalized")
		if err != nil {
			return BeaconBlocksResponseJSON{}, err
		}

		if !resp.Finalized {
			return BeaconBlocksResponseJSON{}, fmt.Errorf("block is not finalized")
		}

		return resp, nil
	})
}
