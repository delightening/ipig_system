# ============================================
# Stage 1: Install dependencies
# ============================================
FROM node:22-alpine AS deps

# Security Plan D: always upgrade base image packages
RUN apk update && apk upgrade --no-cache

WORKDIR /app

COPY package.json ./
RUN npm install --omit=dev --ignore-scripts && \
    npm rebuild sharp

# ============================================
# Stage 2: Production image
# ============================================
FROM node:22-alpine AS production

# Security Plan D: always upgrade base image packages
RUN apk update && apk upgrade --no-cache

# Non-root user for security
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY src ./src
COPY package.json ./

# Set ownership
RUN chown -R appuser:appgroup /app

USER appuser

ENV NODE_ENV=production
ENV PORT=3100

EXPOSE 3100

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1:3100/health || exit 1

CMD ["node", "src/server.js"]
