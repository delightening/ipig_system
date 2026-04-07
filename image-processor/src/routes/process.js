'use strict';

const imageService = require('../services/image');
const config = require('../config');

// SEC-M10: 驗證 operations 參數 schema，防止 DoS 攻擊
function validateOperations(operations) {
  if (typeof operations !== 'object' || operations === null) {
    throw new Error('operations must be an object');
  }
  if (operations.resize !== undefined) {
    const { width, height } = operations.resize;
    if (width !== undefined && (typeof width !== 'number' || width < 1 || width > 8000 || !Number.isFinite(width))) {
      throw new Error('resize.width must be an integer between 1 and 8000');
    }
    if (height !== undefined && (typeof height !== 'number' || height < 1 || height > 8000 || !Number.isFinite(height))) {
      throw new Error('resize.height must be an integer between 1 and 8000');
    }
  }
}

async function processRoutes(fastify, opts) {
  const { internalAuthHook, concurrencyLimiter } = opts;

  fastify.post('/process', {
    preHandler: internalAuthHook,
  }, async (request, reply) => {
    return concurrencyLimiter(async () => {
      const data = await request.file();
      if (!data) {
        return reply.code(400).send({ error: 'No file uploaded' });
      }

      const buffer = await data.toBuffer();
      if (buffer.length === 0) {
        return reply.code(400).send({ error: 'Empty file' });
      }

      const maxBytes = config.maxImageSizeMb * 1024 * 1024;
      if (buffer.length > maxBytes) {
        return reply.code(413).send({
          error: `File size ${buffer.length} exceeds limit of ${maxBytes} bytes`,
        });
      }

      // Parse operations from the 'operations' field (JSON string)
      let operations = {};
      if (data.fields.operations) {
        try {
          const opsValue = data.fields.operations.value;
          operations = JSON.parse(opsValue);
          // SEC-M10: 驗證 operations schema
          validateOperations(operations);
        } catch (err) {
          return reply.code(400).send({ error: `Invalid operations: ${err.message}` });
        }
      }

      try {
        // Validate image
        const metadata = await imageService.getMetadata(buffer);

        // Process image
        const result = await imageService.process(buffer, operations);

        return reply.send({
          metadata,
          output: {
            data: result.data.toString('base64'),
            info: result.info,
          },
        });
      } catch (err) {
        const isValidation = err.message.includes('bomb')
          || err.message.includes('Invalid image')
          || err.message.includes('Unsupported');

        const status = isValidation ? 400 : 500;
        return reply.code(status).send({ error: err.message });
      }
    });
  });
}

module.exports = processRoutes;
