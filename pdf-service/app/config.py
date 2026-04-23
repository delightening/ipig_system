import os


class Config:
    port: int = int(os.environ.get("PORT", "3200"))
    gotenberg_url: str = os.environ.get("GOTENBERG_URL", "http://gotenberg:3000").rstrip("/")
    internal_token: str = os.environ.get("PDF_SERVICE_TOKEN", "")
    gotenberg_timeout_seconds: float = float(os.environ.get("GOTENBERG_TIMEOUT_SECS", "60"))


config = Config()
