'use strict';

async function healthRoutes(fastify) {
  fastify.get('/health', async () => ({
    status: 'ok',
    version: '1.0.0',
    timestamp: new Date().toISOString(),
  }));
}

module.exports = healthRoutes;
