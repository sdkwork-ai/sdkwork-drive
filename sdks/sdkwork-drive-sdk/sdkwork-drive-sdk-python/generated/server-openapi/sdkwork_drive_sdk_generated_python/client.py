from .http_client import HttpClient, SdkConfig
from .api.drive import DriveApi


class SdkworkCustomClient:
    """sdkwork-drive-sdk SDK Client."""

    def __init__(self, config: SdkConfig):
        self._client = HttpClient(config)
        self.drive: DriveApi

        # Initialize API modules
        self.drive = DriveApi(self._client)

    def set_api_key(self, api_key: str) -> 'SdkworkCustomClient':
        """Set API key for authentication."""
        self._client.set_api_key(api_key)
        return self

    def set_auth_token(self, token: str) -> 'SdkworkCustomClient':
        """Set auth token for authentication."""
        self._client.set_auth_token(token)
        return self

    def set_access_token(self, token: str) -> 'SdkworkCustomClient':
        """Set access token for authentication."""
        self._client.set_access_token(token)
        return self

    def set_header(self, key: str, value: str) -> 'SdkworkCustomClient':
        """Set custom header."""
        self._client.set_header(key, value)
        return self

    @property
    def http(self) -> HttpClient:
        """Get the underlying HTTP client."""
        return self._client


def create_client(config: SdkConfig) -> SdkworkCustomClient:
    """Create a new SDK client instance."""
    return SdkworkCustomClient(config)
