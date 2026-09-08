// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"context"
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	pb "google.golang.org/protobuf/proto"

	msgv1 "cosmossdk.io/api/cosmos/msg/v1"
	reflectionv1 "cosmossdk.io/api/cosmos/reflection/v1"

	abci "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/interchaintest/v11/chain/cosmos"
)

var queryReqToPath = make(map[string]string)

func populateQueryReqToPath(ctx context.Context, chain *cosmos.CosmosChain) error {
	resp, err := queryFileDescriptors(ctx, chain)
	if err != nil {
		return err
	}

	for _, fileDescriptor := range resp.Files {
		for _, service := range fileDescriptor.GetService() {
			// Skip services that are annotated with the "cosmos.msg.v1.service" option.
			if ext := pb.GetExtension(service.GetOptions(), msgv1.E_Service); ext != nil && ext.(bool) {
				continue
			}

			for _, method := range service.GetMethod() {
				// trim the first character from input which is a dot
				queryReqToPath[method.GetInputType()[1:]] = "/" + fileDescriptor.GetPackage() + "." + service.GetName() + "/" + method.GetName()
			}
		}
	}

	return nil
}

func ABCIQuery(ctx context.Context, chain *cosmos.CosmosChain, req *abci.RequestQuery) (*abci.ResponseQuery, error) {
	// Create a connection to the gRPC server.
	path := "/cosmos.base.tendermint.v1beta1.Service/ABCIQuery"
	grpcConn, err := grpc.Dial(
		chain.GetHostGRPCAddress(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		retryConfig(),
	)
	if err != nil {
		return &abci.ResponseQuery{}, err
	}

	defer grpcConn.Close()

	resp := &abci.ResponseQuery{}
	err = grpcConn.Invoke(ctx, path, req, resp)
	if err != nil {
		return &abci.ResponseQuery{}, err
	}

	return resp, nil
}

// Queries the chain with a query request and deserializes the response to T
func GRPCQuery[T any](ctx context.Context, chain *cosmos.CosmosChain, req proto.Message, opts ...grpc.CallOption) (*T, error) {
	path, ok := queryReqToPath[proto.MessageName(req)]
	if !ok {
		return nil, fmt.Errorf("no path found for %s", proto.MessageName(req))
	}

	// Create a connection to the gRPC server.
	grpcConn, err := grpc.Dial(
		chain.GetHostGRPCAddress(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		retryConfig(),
	)
	if err != nil {
		return nil, err
	}

	defer grpcConn.Close()

	resp := new(T)
	err = grpcConn.Invoke(ctx, path, req, resp, opts...)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func queryFileDescriptors(ctx context.Context, chain *cosmos.CosmosChain) (*reflectionv1.FileDescriptorsResponse, error) {
	// Create a connection to the gRPC server.
	grpcConn, err := grpc.Dial(
		chain.GetHostGRPCAddress(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		retryConfig(),
	)
	if err != nil {
		return nil, err
	}

	defer grpcConn.Close()

	resp := new(reflectionv1.FileDescriptorsResponse)
	err = grpcConn.Invoke(
		ctx, reflectionv1.ReflectionService_FileDescriptors_FullMethodName,
		&reflectionv1.FileDescriptorsRequest{}, resp,
	)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func retryConfig() grpc.DialOption {
	policy := `{
            "methodConfig": [{
                "name": [{}],
                "retryPolicy": {
                    "MaxAttempts": 4,
                    "InitialBackoff": ".01s",
                    "MaxBackoff": ".01s",
                    "BackoffMultiplier": 1.0,
                    "RetryableStatusCodes": [
						"CANCELLED",
						"UNKNOWN",
						"DEADLINE_EXCEEDED",
						"NOT_FOUND",
						"ALREADY_EXISTS",
						"PERMISSION_DENIED",
						"RESOURCE_EXHAUSTED",
						"FAILED_PRECONDITION",
						"ABORTED",
						"OUT_OF_RANGE",
						"UNIMPLEMENTED",
						"INTERNAL",
						"UNAVAILABLE",
						"DATA_LOSS",
						"UNAUTHENTICATED"
				    ]
                }
            }]
        }`

	return grpc.WithDefaultServiceConfig(policy)
}
