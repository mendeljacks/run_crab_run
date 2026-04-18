FROM node:22-slim AS builder
WORKDIR /usr/src/app
RUN npm install -g pnpm
COPY ./runner/package.json ./runner/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY ./runner/tsconfig.json ./
COPY ./runner/src ./src
RUN pnpm run build

FROM node:22-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/dist ./dist
COPY --from=builder /usr/src/app/package.json ./
COPY --from=builder /usr/src/app/node_modules ./node_modules
ENV PORT=4000
CMD ["node", "dist/index.js"]