# Image Processor Microservice

Independent image processing microservice for ipig_system, based on Security Plan D.

## Architecture

All image decoding/processing is isolated in this service, which runs on a regularly-upgradeable
base image (`node:22-alpine` with `apk upgrade`). The Rust backend never decodes image content
directly; it delegates to this service via HTTP.

## API

### GET /health

Health check endpoint.

**Response:**
```json
{ "status": "ok", "version": "1.0.0", "timestamp": "2026-03-23T..." }
```

### POST /process

Process an image file. Accepts multipart form data.

**Fields:**
- `file` (required): Image file (JPEG, PNG, WebP, GIF, AVIF, TIFF)
- `operations` (optional): JSON string with processing instructions

**Operations format:**
```json
{
  "resize": { "width": 800, "height": 600, "fit": "inside" },
  "format": "webp",
  "quality": 85
}
```

**Response:**
```json
{
  "metadata": { "width": 4000, "height": 3000, "format": "jpeg", ... },
  "output": {
    "data": "<base64-encoded image>",
    "info": { "width": 800, "height": 600, "format": "webp", "size": 12345 }
  }
}
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3100` | Server listen port |
| `MAX_IMAGE_SIZE_MB` | `30` | Maximum upload size in MB |
| `MAX_CONCURRENCY` | `4` | Maximum concurrent processing tasks |
| `MAX_PIXELS` | `100000000` | Maximum pixel count (decompression bomb protection) |

## Security

- EXIF/metadata is always stripped from output images
- Decompression bomb protection via pixel count limit
- Non-root container user
- Internal network only (not exposed to external traffic)
- Base image upgraded on every build (`apk update && apk upgrade`)
