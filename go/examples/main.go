// Worked example for the Quicknode Go SDK.
//
// Build the binding first (`just go-build` from the repo root), then run:
//
//	QN_SDK__API_KEY=your-key go run ./go/examples
//
// Requires cgo and the statically-linked native library that `just go-build`
// produces under go/quicknode_sdk/lib/<goos>_<goarch>/.
package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"os"

	qn "github.com/quicknode/sdk/go/quicknode_sdk"
)

func main() {
	apiKey := os.Getenv("QN_SDK__API_KEY")
	if apiKey == "" {
		log.Fatal("set QN_SDK__API_KEY")
	}

	client, err := qn.NewQuicknodeSdkClient(apiKey)
	if err != nil {
		log.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	// ── Admin: list endpoints ────────────────────────────────────────────────
	endpoints, err := client.Admin().GetEndpoints(qn.GetEndpointsRequest{})
	if err != nil {
		log.Fatalf("get endpoints: %v", err)
	}
	fmt.Printf("endpoints: %d\n", len(endpoints.Data))
	for _, ep := range endpoints.Data {
		fmt.Printf("  %s  %s/%s  %s\n", ep.Id, ep.Chain, ep.Network, ep.HttpUrl)
	}

	// ── Streams: the destination is a discriminated union ─────────────────────
	// Construct a webhook destination; uniffi marshals the union natively.
	streamParams := qn.CreateStreamParams{
		Name:       "example-stream",
		Region:     qn.StreamRegionUsaEast,
		Network:    "ethereum-mainnet",
		Dataset:    qn.StreamDatasetBlock,
		StartRange: 0,
		EndRange:   -1,
		DestinationAttributes: qn.DestinationAttributesWebhook{
			Field0: qn.WebhookAttributes{
				Url:              "https://example.com/hook",
				MaxRetry:         3,
				RetryIntervalSec: 10,
				PostTimeoutSec:   30,
			},
		},
		DatasetBatchSize:    1,
		ElasticBatchEnabled: false,
	}
	// Commented out so the example is read-only by default — uncomment to create:
	// stream, err := client.Streams().CreateStream(streamParams)
	_ = streamParams

	streams, err := client.Streams().ListStreams(qn.ListStreamsParams{})
	if err != nil {
		log.Fatalf("list streams: %v", err)
	}
	fmt.Printf("streams: %d\n", len(streams.Data))

	// ── SQL: result rows are arbitrary JSON, surfaced as []string ─────────────
	result, err := client.Sql().Query(qn.QueryParams{
		Query:     "SELECT number FROM eth_mainnet.blocks LIMIT 3",
		ClusterId: "eth-mainnet",
	})
	if err != nil {
		// SQL access is plan-gated; treat as non-fatal for the example.
		fmt.Printf("sql query skipped: %v\n", err)
		return
	}
	fmt.Printf("sql rows: %d\n", result.Rows)
	for _, raw := range result.Data {
		var row map[string]any
		if err := json.Unmarshal([]byte(raw), &row); err != nil {
			log.Fatalf("decode row: %v", err)
		}
		fmt.Printf("  %v\n", row)
	}

	// ── Typed errors: extract the variant with errors.As ─────────────────────
	if _, err := client.Admin().ShowEndpoint("does-not-exist"); err != nil {
		var apiErr *qn.QuicknodeErrorApi
		if errors.As(err, &apiErr) {
			fmt.Printf("expected API error: status=%d\n", apiErr.Status)
		} else {
			fmt.Printf("error: %v\n", err)
		}
	}
}
