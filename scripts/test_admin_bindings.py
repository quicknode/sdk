"""Run native Admin binding examples against fabricated localhost responses.

Build each binding first. Select a subset with --language python/node/ruby.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parents[1]
PATTERN = {"method": [{"prefix": "eth_"}], "anything-but": [None, {"nested_key": [True, 7.5]}]}
TOTALS = {"requests": 8.5, "bytes": 10.5, "rpc_credits": 7.5, "data_credits": 3.5, "credits": 11.0}
SECURITY = {
    "request_filters": [{"id": "filter", "method": PATTERN["method"], "params": {"anything-but": PATTERN["anything-but"]}}],
    "blocked_ips": [{"id": "ip", "ip": "192.0.2.0/24"}],
    "options": {"hsts": True, "cors": False, "responseLogging": True},
}
HEIGHT = "9007199254740993"
PAYLOAD = ' { "method" : "eth_sendRawTransaction" }\n'


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.handle_request()

    def do_POST(self):
        self.handle_request()

    def do_PUT(self):
        self.handle_request()

    def do_PATCH(self):
        self.handle_request()

    def handle_request(self):
        try:
            status, payload = self.response()
        except Exception as error:
            self.server.failures.append(str(error))
            status, payload = 500, {"error": str(error)}
        body = b"" if status == 204 else json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def response(self):
        url = urlsplit(self.path)
        query = parse_qs(url.query)
        path = url.path
        method = self.command
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"]))) if self.headers.get("Content-Length") else None
        assert self.headers.get("x-api-key") == "fixture-key", "missing fixture authentication"
        self.server.calls.append((method, path))
        if path == "/endpoints/missing/security":
            return 404, {"error": "fixture missing endpoint"}
        if path == "/endpoints/12/security/request_filters" and method == "POST":
            assert body == {"pattern": PATTERN}, body
            return 200, {"data": {"id": "filter", "pattern": PATTERN}}
        if path == "/endpoints/12/security/request_filters/filter" and method == "PUT":
            assert body == {"pattern": PATTERN}, body
            return 204, None
        if path == "/endpoints/12/security/request_filters" and method == "GET":
            return 200, {"data": [{"id": "filter", "pattern": PATTERN, "method_whitelist": False}]}
        if path == "/endpoints/12/security":
            return 200, {"data": SECURITY}
        if path == "/endpoints/12":
            return 200, {"data": {"id": "12", "chain": "ethereum", "network": "mainnet", "http_url": "https://example.invalid", "security": SECURITY}}
        if path == "/endpoints/12/security/tokens" and method == "POST":
            return 200, {"data": {"id": "token", "token": "fabricated-token"}}
        if path == "/endpoints/12/method-rate-limits/rule" and method == "PATCH":
            assert body == {"limit_by_ip": False}, body
            return 200, {"data": {"id": "rule", "interval": "second", "methods": ["eth_chainId"], "rate": 5, "status": "enabled", "created": "2026-10-01T00:00:00Z", "limit_by_ip": False}}
        if path == "/usage/rpc/summary":
            data = {"from": "2026-10-01T00:00:00Z", "to": "2026-10-02T00:00:00Z", "totals": TOTALS}
            if "cursor" in query:
                assert query == {"cursor": ["opaque-cursor"]}, query
                data["breakdown"] = []
            else:
                assert query == {"last": ["1d"], "group_by[]": ["method", "endpoint"]}, query
                data["breakdown"] = [{**TOTALS, "requests": None, "method": None}]
                data["next_cursor"] = "opaque-cursor"
            return 200, {"data": data}
        if path == "/metrics/traffic/summary":
            assert query == {"last": ["1d"], "group_by[]": ["method"]}, query
            totals = {"successful_requests": 7.5, "bytes": 10.5, "request_credits": 5.5, "bytes_credits": 3.5, "credits": 9.0, "active_methods": 1.0}
            return 200, {"data": {"from": "2026-10-01T00:00:00Z", "to": "2026-10-02T00:00:00Z", "totals": totals, "breakdown": [{**totals, "successful_requests": None, "method": None}]}}
        if path == "/transactions":
            assert query == {"last": ["1d"]}, query
            row = {"network": "ethereum-mainnet", "transaction_hash": "hash", "first_submitted_at": "2026-10-01T00:00:00Z", "last_submitted_at": "2026-10-01T00:00:01Z", "state": "succeeded", "attempt_count": 1, "endpoint_ids": ["fixture-uuid"], "methods": ["eth_sendRawTransaction"], "archive": False, "height": HEIGHT}
            return 200, {"data": {"from": "2026-10-01T00:00:00Z", "to": "2026-10-02T00:00:00Z", "transactions": [row]}}
        if path == "/transactions/ethereum-mainnet/hash":
            assert query.get("include_payloads") in [["false"], ["true"]], query
            if query["include_payloads"] == ["false"]:
                return 200, {"data": {"exists": False, "data": None}}
            return 200, {"data": {"exists": True, "data": {"network": "ethereum-mainnet", "transaction_hash": "hash", "first_submitted_at": "2026-10-01T00:00:00Z", "last_submitted_at": "2026-10-01T00:00:01Z", "state": "succeeded", "height": HEIGHT, "attempts": [{"submitted_at": "2026-10-01T00:00:00Z", "endpoint_id": "fixture-uuid", "method": "eth_sendRawTransaction", "request_id": "request", "payloads": {"request": PAYLOAD, "response": "plain text"}}]}}}
        raise AssertionError(f"Unexpected request: {method} {self.path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--language", choices=["python", "node", "ruby"], action="append")
    args = parser.parse_args()
    commands = {
        "python": [str(ROOT / ".venv/bin/python"), "python/examples/admin_mock.py"],
        "node": ["node", "npm/examples/admin_mock.js"],
        "ruby": [os.environ.get("RUBY", "ruby"), "-I", "ruby/lib", "ruby/examples/admin_mock.rb"],
    }
    with ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server:
        server.failures = []
        server.calls = []
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for language in args.language or commands:
                server.calls.clear()
                env = dict(os.environ, ADMIN_MOCK_URL=f"http://127.0.0.1:{server.server_port}/")
                subprocess.run(commands[language], cwd=ROOT, env=env, check=True, timeout=60)
                assert not server.failures, server.failures
                assert len(server.calls) == 14, (language, server.calls)
                print(f"{language}: 14 localhost Admin requests verified", flush=True)
        finally:
            server.shutdown()
            thread.join()


if __name__ == "__main__":
    main()
