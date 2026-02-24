# Runtime stage
FROM nginx:1.27-alpine

# Security upgrade
RUN apk update && apk upgrade && rm -rf /var/cache/apk/*

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy build files from host (pre-built via npm run build)
COPY dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
