"""Smoke test all 11 print-pdf templates via /api/sample + /api/render.

Verifies the service can produce valid PDFs (magic bytes %PDF-) for every
template after a deploy. Output: PASS/FAIL summary per template, exit 1 on
any failure.

Reads PDF_SERVICE_TOKEN / PDF_SERVICE_TOKEN_FILE so it works post-R55-3 once
/api/render is protected. Default: load from
`secrets/pdf_service_token.txt` (repo-local dev secret).
"""

import json
import os
import sys
import urllib.request
import urllib.error
from pathlib import Path

BASE = "http://127.0.0.1:9210"
TEMPLATES = [
    "audit_log",
    "aup_protocol",
    "blood_test",
    "medical_record",
    "pig_approval",
    "review_reply",
    "review_result",
    "surgery",
    "vet_patrol",
    "vet_patrol_report",
    "warehouse",
]


def _resolve_token() -> str:
    """Reuse same dual-mode resolution as main.py / backend Config::read_secret."""
    tok = os.environ.get("PDF_SERVICE_TOKEN", "").strip()
    if tok:
        return tok
    file_env = os.environ.get("PDF_SERVICE_TOKEN_FILE", "").strip()
    candidates = [file_env] if file_env else []
    candidates.append(str(Path(__file__).resolve().parents[3] / "secrets" / "pdf_service_token.txt"))
    for p in candidates:
        try:
            return Path(p).read_text(encoding="utf-8").strip()
        except OSError:
            continue
    return ""


TOKEN = _resolve_token()
HEADERS = {"Content-Type": "application/json"}
if TOKEN:
    HEADERS["X-Internal-Token"] = TOKEN


def fetch_json(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=10) as r:
        return json.loads(r.read())


def post_json(url: str, payload: dict, timeout: int = 30) -> bytes:
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers=HEADERS,
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()


def main() -> int:
    out_dir = Path(__file__).parent / "_smoke_out"
    out_dir.mkdir(exist_ok=True)
    failures: list[str] = []

    print(f"smoke testing {len(TEMPLATES)} templates against {BASE}")
    print(f"X-Internal-Token: {'set (len=' + str(len(TOKEN)) + ')' if TOKEN else 'NOT set'}")
    for t in TEMPLATES:
        try:
            sample = fetch_json(f"{BASE}/api/sample/{t}")
            pdf = post_json(f"{BASE}/api/render/{t}", sample)
            if not pdf.startswith(b"%PDF-"):
                failures.append(f"{t}: bad magic ({pdf[:8]!r})")
                print(f"  FAIL {t}: not a PDF (first bytes: {pdf[:32]!r})")
                continue
            (out_dir / f"{t}.pdf").write_bytes(pdf)
            print(f"  OK   {t} ({len(pdf):>7} bytes)")
        except urllib.error.HTTPError as e:
            body = e.read().decode("utf-8", errors="replace")[:200]
            failures.append(f"{t}: HTTP {e.code} {body}")
            print(f"  FAIL {t}: HTTP {e.code} — {body}")
        except Exception as e:
            failures.append(f"{t}: {type(e).__name__}: {e}")
            print(f"  FAIL {t}: {type(e).__name__}: {e}")

    print()
    print(f"output dir: {out_dir}")
    if failures:
        print(f"\n{len(failures)} FAILURE(S):")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(f"\nall {len(TEMPLATES)} templates PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
