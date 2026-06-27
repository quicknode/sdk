package quicknode_sdk

import (
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
)

// Proves the full FFI path end to end: the test statically links
// libquicknode_sdk.a (via cgo_link_*.go), constructs the SDK pointed at a mock
// HTTP server, and asserts that an async core call bridged through block_on
// returns a decoded response. This is the Stage 1 validation that the whole Go
// binding design hinges on.
func TestGetEndpointsRoundTrip(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/endpoints" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}
		if got := r.Header.Get("x-api-key"); got != "test-key" {
			t.Errorf("x-api-key = %q, want test-key", got)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"data": [
				{
					"id": "ep-1",
					"name": "spring-cool-sky",
					"label": "my endpoint",
					"status": "active",
					"chain": "ethereum",
					"network": "mainnet",
					"is_dedicated": false,
					"is_flat_rate": true,
					"http_url": "https://example.quiknode.pro/abc",
					"wss_url": null,
					"tags": [{"tag_id": 7, "label": "prod"}],
					"is_multichain": false
				}
			],
			"pagination": {"total": 1, "limit": 20, "offset": 0},
			"error": null
		}`))
	}))
	defer server.Close()

	client, err := QuicknodeSdkClientNewWithBaseUrls("test-key", BaseUrlOverrides{Admin: strPtr(server.URL + "/")})
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	resp, err := client.Admin().GetEndpoints(GetEndpointsRequest{})
	if err != nil {
		t.Fatalf("GetEndpoints: %v", err)
	}

	if len(resp.Data) != 1 {
		t.Fatalf("got %d endpoints, want 1", len(resp.Data))
	}
	ep := resp.Data[0]
	if ep.Id != "ep-1" {
		t.Errorf("Id = %q, want ep-1", ep.Id)
	}
	if ep.Chain != "ethereum" || ep.Network != "mainnet" {
		t.Errorf("chain/network = %q/%q, want ethereum/mainnet", ep.Chain, ep.Network)
	}
	if ep.Label == nil || *ep.Label != "my endpoint" {
		t.Errorf("Label = %v, want \"my endpoint\"", ep.Label)
	}
	if len(ep.Tags) != 1 || ep.Tags[0].Label != "prod" {
		t.Errorf("Tags = %+v, want one tag labeled prod", ep.Tags)
	}
	if resp.Pagination == nil || resp.Pagination.Total != 1 {
		t.Errorf("Pagination = %+v, want total 1", resp.Pagination)
	}
}

// Proves SdkError -> typed Go error mapping survives the boundary: a 401 should
// surface as QuicknodeErrorApi carrying the status and body.
func TestGetEndpointsApiError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		w.Write([]byte(`{"error":"invalid api key"}`))
	}))
	defer server.Close()

	client, err := QuicknodeSdkClientNewWithBaseUrls("bad-key", BaseUrlOverrides{Admin: strPtr(server.URL + "/")})
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	_, err = client.Admin().GetEndpoints(GetEndpointsRequest{})
	if err == nil {
		t.Fatal("expected an error, got nil")
	}
	var apiErr *QuicknodeErrorApi
	if !errors.As(err, &apiErr) {
		t.Fatalf("error = %v, want a QuicknodeErrorApi variant", err)
	}
	if apiErr.Status != 401 {
		t.Errorf("Status = %d, want 401", apiErr.Status)
	}
	if apiErr.Body == "" {
		t.Error("Body is empty, want the raw error response")
	}
}

// Proves the DestinationAttributes discriminated union marshals correctly in
// BOTH directions: a Webhook variant is lowered into the request, and the
// Stream response (which also carries the union) is lifted back. This is the
// codegen path uniffi handles natively but napi/pyo3 cannot.
func TestCreateStreamWithWebhookDestination(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"id": "stream-1",
			"name": "my-stream",
			"status": "active",
			"created_at": "2026-01-01T00:00:00Z",
			"updated_at": "2026-01-01T00:00:00Z",
			"sequence": 0,
			"network": "ethereum-mainnet",
			"dataset": "block",
			"region": "usa_east",
			"start_range": 1,
			"end_range": -1,
			"dataset_batch_size": 1,
			"elastic_batch_enabled": false,
			"destination": "webhook",
			"destination_attributes": {
				"url": "https://example.com/hook",
				"max_retry": 3,
				"retry_interval_sec": 10,
				"post_timeout_sec": 30
			}
		}`))
	}))
	defer server.Close()

	client, err := QuicknodeSdkClientNewWithBaseUrls("k", BaseUrlOverrides{Streams: strPtr(server.URL + "/")})
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	params := CreateStreamParams{
		Name:       "my-stream",
		Region:     StreamRegionUsaEast,
		Network:    "ethereum-mainnet",
		Dataset:    StreamDatasetBlock,
		StartRange: 1,
		EndRange:   -1,
		DestinationAttributes: DestinationAttributesWebhook{
			Field0: WebhookAttributes{
				Url:              "https://example.com/hook",
				MaxRetry:         3,
				RetryIntervalSec: 10,
				PostTimeoutSec:   30,
			},
		},
		DatasetBatchSize:    1,
		ElasticBatchEnabled: false,
	}

	stream, err := client.Streams().CreateStream(params)
	if err != nil {
		t.Fatalf("CreateStream: %v", err)
	}
	if stream.Id != "stream-1" {
		t.Errorf("Id = %q, want stream-1", stream.Id)
	}
	// The response carries the union back; assert it lifted into the Webhook
	// variant. The field is optional in core, so Go surfaces it as a pointer to
	// the interface — deref before the type switch.
	if stream.DestinationAttributes == nil {
		t.Fatal("DestinationAttributes is nil, want a Webhook variant")
	}
	wh, ok := (*stream.DestinationAttributes).(DestinationAttributesWebhook)
	if !ok {
		t.Fatalf("DestinationAttributes = %T, want DestinationAttributesWebhook", *stream.DestinationAttributes)
	}
	if wh.Field0.Url != "https://example.com/hook" {
		t.Errorf("webhook url = %q, want https://example.com/hook", wh.Field0.Url)
	}
}

// Proves the serde_json::Value custom_type path: SQL query result rows are
// arbitrary JSON, marshaled to Go as []string (each a JSON document the caller
// unmarshals). Validates the uniffi custom_type! registration end to end.
func TestSqlQueryJsonRows(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"meta": [{"name": "n", "type": "UInt64"}],
			"data": [{"n": 1}, {"n": 2}],
			"rows": 2,
			"rows_before_limit_at_least": 2,
			"statistics": {"elapsed": 0.01, "rows_read": 2, "bytes_read": 16},
			"credits": 1
		}`))
	}))
	defer server.Close()

	client, err := QuicknodeSdkClientNewWithBaseUrls("k", BaseUrlOverrides{Sql: strPtr(server.URL + "/")})
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	resp, err := client.Sql().Query(QueryParams{Query: "SELECT 1", ClusterId: "eth-mainnet"})
	if err != nil {
		t.Fatalf("Query: %v", err)
	}
	if resp.Rows != 2 || len(resp.Data) != 2 {
		t.Fatalf("rows=%d data=%d, want 2/2", resp.Rows, len(resp.Data))
	}
	// Each row is a JSON string; unmarshal the first and check it.
	var row map[string]int
	if err := json.Unmarshal([]byte(resp.Data[0]), &row); err != nil {
		t.Fatalf("unmarshal row: %v (raw=%q)", err, resp.Data[0])
	}
	if row["n"] != 1 {
		t.Errorf("row[n] = %d, want 1", row["n"])
	}
}

func strPtr(s string) *string { return &s }
