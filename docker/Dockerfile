# MineSweeper - 單元測試用 Docker 映像
# Tauri GUI 應用需在 macOS 本機編譯執行，此映像僅執行核心邏輯單元測試
FROM rust:1-slim

WORKDIR /app

# 先複製 manifest 並預先抓取相依套件，讓原始碼變動時可沿用快取層
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml core/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
RUN mkdir -p core/src frontend/src src-tauri/src \
  && touch core/src/lib.rs \
  && echo 'fn main() {}' > frontend/src/main.rs \
  && echo 'fn main() {}' > src-tauri/src/main.rs \
  && touch src-tauri/src/lib.rs \
  && cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"

COPY . .

CMD ["cargo", "test", "--locked", "-p", "minesweeper-core"]
