'use strict';

const fastify = require('fastify');
const multipart = require('@fastify/multipart');
const config = require('./config');
const healthRoutes = require('./routes/health');
const processRoutes = require('./routes/process');

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
  await app.register(processRoutes);

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
