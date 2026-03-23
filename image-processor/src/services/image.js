'use strict';

const sharp = require('sharp');
const config = require('../config');

/**
 * Validate image by attempting to decode metadata with sharp.
 * Also checks for decompression bomb (pixel count exceeds limit).
 *
 * @param {Buffer} buffer - Raw image data
 * @returns {Promise<import('sharp').Metadata>} Validated metadata
 */
async function validate(buffer) {
  const metadata = await sharp(buffer, { limitInputPixels: config.maxPixels })
    .metadata();

  if (!metadata.width || !metadata.height) {
    throw new Error('Invalid image: unable to read dimensions');
  }

  const pixels = metadata.width * metadata.height;
  if (pixels > config.maxPixels) {
    throw new Error(
      `Decompression bomb detected: ${pixels} pixels exceeds limit of ${config.maxPixels}`
    );
  }

  return metadata;
}

/**
 * Get image metadata without processing.
 *
 * @param {Buffer} buffer - Raw image data
 * @returns {Promise<object>} Simplified metadata
 */
async function getMetadata(buffer) {
  const meta = await validate(buffer);
  return {
    width: meta.width,
    height: meta.height,
    format: meta.format,
    channels: meta.channels,
    size: buffer.length,
    hasAlpha: meta.hasAlpha || false,
  };
}

/**
 * Process image with given operations.
 * Always strips EXIF/metadata for security.
 *
 * @param {Buffer} buffer - Raw image data
 * @param {object} operations - Processing operations
 * @param {object} [operations.resize] - Resize options
 * @param {number} [operations.resize.width] - Target width
 * @param {number} [operations.resize.height] - Target height
 * @param {string} [operations.resize.fit] - Fit mode (cover, contain, fill, inside, outside)
 * @param {string} [operations.format] - Output format (jpeg, png, webp, avif)
 * @param {number} [operations.quality] - Output quality (1-100)
 * @returns {Promise<{data: Buffer, info: object}>} Processed image
 */
async function process(buffer, operations = {}) {
  await validate(buffer);

  let pipeline = sharp(buffer, { limitInputPixels: config.maxPixels });

  // Always strip metadata (EXIF, IPTC, XMP) for security
  pipeline = pipeline.rotate(); // auto-rotate based on EXIF before stripping

  // Resize
  if (operations.resize) {
    const { width, height, fit } = operations.resize;
    pipeline = pipeline.resize({
      width: width || undefined,
      height: height || undefined,
      fit: fit || 'inside',
      withoutEnlargement: true,
    });
  }

  // Format conversion
  const format = operations.format || 'jpeg';
  const quality = operations.quality || 85;

  switch (format) {
    case 'jpeg':
    case 'jpg':
      pipeline = pipeline.jpeg({ quality, mozjpeg: true });
      break;
    case 'png':
      pipeline = pipeline.png({ quality, compressionLevel: 9 });
      break;
    case 'webp':
      pipeline = pipeline.webp({ quality });
      break;
    case 'avif':
      pipeline = pipeline.avif({ quality });
      break;
    default:
      throw new Error(`Unsupported output format: ${format}`);
  }

  const { data, info } = await pipeline.toBuffer({ resolveWithObject: true });

  return {
    data,
    info: {
      width: info.width,
      height: info.height,
      format: info.format,
      size: info.size,
      channels: info.channels,
    },
  };
}

module.exports = { validate, getMetadata, process };
