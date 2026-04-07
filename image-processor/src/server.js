'use strict';

const fastify = require('fastify');
const multipart = require('@fastify/multipart');
const pLimit = require('p-limit');
const config = require('./config');
const healthRoutes = require('./routes/health');
const processRoutes = require('./routes/process');

// SEC-C6: 全域並發限制，防止 DoS
const concurrencyLimiter = pLimit(config.maxConcurrency);

// SEC-C6: 內部服務認證 hook（X-Internal-Token）
async function internalAuthHook(request, reply) {
  if (!config.internalToken) {
    // 未設定 token 時拒絕所有請求，防止意外暴露
    reply.code(503).send({ error: 'Service not configured: IMAGE_PROCESSOR_TOKEN not set' });
    return;
  }
  const provided = request.headers['x-internal-token'] || '';
  // 使用 timingSafeEqual 防止 timing attack
  const { timingSafeEqual } = require('crypto');
  const expected = Buffer.from(config.internalToken);
  const actual = Buffer.from(provided.padEnd(config.internalToken.length, '\0').slice(0, config.internalToken.length));
  if (expected.length !== actual.length || !timingSafeEqual(expected, actual)) {
    reply.code(401).send({ error: 'Unauthorized' });
    return;
  }
}

async function build() {
  const app = fastify({
    logger: true,
    bodyLimit: config.maxImageSizeMb * 1024 * 1024,
  });

  await app.register(multipart, {
    limits: {
      fileSize: config.maxImageSizeMb * 1024 * 1024,
      files: 1,
    },
  });

  await app.register(healthRoutes);
  await app.register(processRoutes, { internalAuthHook, concurrencyLimiter });

  return app;
}

async function start() {
  const app = await build();

  try {
    await app.listen({ port: config.port, host: '0.0.0.0' });
    app.log.info(`Image processor listening on port ${config.port}`);
  } catch (err) {
    app.log.error(err);
    process.exit(1);
  }
}

start();
