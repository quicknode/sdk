from sdk._core import init as _init, HttpbinClient as _HttpbinClient

__all__ = ["init", "httpbin"]

httpbin: "HttpbinClient | None" = None


class HttpbinClient:
    """Client for httpbin API."""

    def __init__(self) -> None:
        self._client = _HttpbinClient()

    async def get_uuid(self) -> str:
        """Get a UUID from httpbin."""
        return await self._client.get_uuid()


def init(api_key: str) -> None:
    """Initialize the SDK with your API key."""
    global httpbin

    _init(api_key)
    httpbin = HttpbinClient()

