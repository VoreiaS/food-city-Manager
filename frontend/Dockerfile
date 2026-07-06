# Frontend Dockerfile — dev and prod stages.

# -----------------------------------------------------------------------------
# Stage: base
# -----------------------------------------------------------------------------
FROM node:22-alpine AS base
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY . .

# -----------------------------------------------------------------------------
# Stage: development
# -----------------------------------------------------------------------------
FROM base AS development
EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host"]

# -----------------------------------------------------------------------------
# Stage: build (compile TS + bundle Vite)
# -----------------------------------------------------------------------------
FROM base AS build
RUN npm run build

# -----------------------------------------------------------------------------
# Stage: production runtime (nginx static serve)
# -----------------------------------------------------------------------------
FROM nginx:alpine AS production
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
