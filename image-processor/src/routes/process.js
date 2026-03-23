'use strict';

const imageService = require('../services/image');
const config = require('../config');

async function processRoutes(fastify) {
  fastify.post('/process', async (request, reply) => {
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
      } catch {
        return reply.code(400).send({ error: 'Invalid operations JSON' });
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
}

module.exports = processRoutes;
