from fastapi import Depends, FastAPI, HTTPException, Request, status
from fastapi.responses import JSONResponse, Response

from .auth import require_internal_token
from .doc_types import REGISTRY
from .renderer import gotenberg_healthy, html_to_pdf, render_html

app = FastAPI(title="iPig PDF Service", version="0.1.0")


@app.get("/health")
async def health() -> dict:
    gotenberg_ok = await gotenberg_healthy()
    return {
        "status": "ok",
        "gotenberg": "up" if gotenberg_ok else "down",
        "doc_types": sorted(REGISTRY.keys()),
    }


@app.post("/render/{doc_type}", dependencies=[Depends(require_internal_token)])
async def render(doc_type: str, request: Request) -> Response:
    entry = REGISTRY.get(doc_type)
    if entry is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Unknown doc_type: {doc_type}",
        )
    body = await request.json()
    try:
        payload = entry.schema_cls.model_validate(body)
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Invalid payload: {e}",
        ) from e

    html = render_html(entry.template, payload.model_dump())
    pdf_bytes = await html_to_pdf(html)
    filename = entry.filename_fn(payload)
    return Response(
        content=pdf_bytes,
        media_type="application/pdf",
        headers={"Content-Disposition": f'attachment; filename="{filename}"'},
    )


@app.exception_handler(RuntimeError)
async def runtime_error_handler(_: Request, exc: RuntimeError) -> JSONResponse:
    return JSONResponse(status_code=502, content={"detail": str(exc)})
