import hmac

from fastapi import Header, HTTPException, status

from .config import config


async def require_internal_token(x_internal_token: str = Header(default="")) -> None:
    # 呼應 image-processor：未設定 token 時直接拒絕，避免意外暴露
    if not config.internal_token:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Service not configured: PDF_SERVICE_TOKEN not set",
        )
    if not hmac.compare_digest(x_internal_token, config.internal_token):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Unauthorized")
