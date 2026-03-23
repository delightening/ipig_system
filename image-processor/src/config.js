'use strict';

const config = {
  port: parseInt(process.env.PORT, 10) || 3100,
  maxImageSizeMb: parseInt(process.env.MAX_IMAGE_SIZE_MB, 10) || 30,
  maxConcurrency: parseInt(process.env.MAX_CONCURRENCY, 10) || 4,
  maxPixels: parseInt(process.env.MAX_PIXELS, 10) || 100_000_000,
};

module.exports = config;
