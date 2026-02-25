# ============================================
# Stage 1: Build frontend
# ============================================
FROM node:22-alpine AS builder

WORKDIR /app

# Copy package files first for layer caching
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts

# Copy source and build
COPY . .
RUN npm run build

# ============================================
# Stage 2: Serve with nginx
# ============================================
FROM nginx:1.27-alpine

# Security upgrade
RUN apk update && apk upgrade && rm -rf /var/cache/apk/*

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy build output from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://localhost:80/ || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
