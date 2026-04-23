from pathlib import Path

import httpx
from jinja2 import Environment, FileSystemLoader, select_autoescape

from .config import config

TEMPLATES_DIR = Path(__file__).parent / "templates"

_env = Environment(
    loader=FileSystemLoader(str(TEMPLATES_DIR)),
    autoescape=select_autoescape(["html", "xml"]),
    trim_blocks=True,
    lstrip_blocks=True,
)


def render_html(template_name: str, context: dict) -> str:
    return _env.get_template(template_name).render(**context)


async def html_to_pdf(html: str) -> bytes:
    url = f"{config.gotenberg_url}/forms/chromium/convert/html"
    files = {
        "files": ("index.html", html.encode("utf-8"), "text/html"),
    }
    data = {
        "paperWidth": "8.27",
        "paperHeight": "11.7",
        "marginTop": "0",
        "marginBottom": "0",
        "marginLeft": "0",
        "marginRight": "0",
        "printBackground": "true",
    }
    timeout = httpx.Timeout(config.gotenberg_timeout_seconds, connect=5.0)
    async with httpx.AsyncClient(timeout=timeout) as client:
        resp = await client.post(url, files=files, data=data)
    if resp.status_code != 200:
        raise RuntimeError(f"Gotenberg returned {resp.status_code}: {resp.text[:400]}")
    return resp.content


async def gotenberg_healthy() -> bool:
    try:
        async with httpx.AsyncClient(timeout=httpx.Timeout(3.0)) as client:
            resp = await client.get(f"{config.gotenberg_url}/health")
        return resp.status_code == 200
    except httpx.HTTPError:
        return False
