package quicknode_sdk

import (
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

	client, err := QuicknodeSdkClientNewWithAdminBaseUrl("test-key", server.URL+"/")
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	resp, err := client.GetEndpoints(GetEndpointsRequest{})
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

	client, err := QuicknodeSdkClientNewWithAdminBaseUrl("bad-key", server.URL+"/")
	if err != nil {
		t.Fatalf("construct client: %v", err)
	}
	defer client.Destroy()

	_, err = client.GetEndpoints(GetEndpointsRequest{})
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
