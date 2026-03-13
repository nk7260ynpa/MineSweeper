# MineSweeper - CI 測試用 Docker 映像
# 注意：Electron GUI 應用不適合在容器中執行，此 Dockerfile 僅供 CI 測試使用
FROM node:20-slim

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm ci --ignore-scripts

COPY . .

CMD ["npm", "test"]
