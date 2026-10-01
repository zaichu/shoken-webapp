# CI と同じ Ubuntu 24.04 で Playwright を動かすためのイメージ。
# フォント・依存パッケージの一覧は含めない。ビルド時に --build-arg APT_PACKAGES で
# .github/workflows/frontend.yml から読み取った値を渡す(二重管理しない)。
FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG NODE_MAJOR=22
ARG APT_PACKAGES=""
ARG PLAYWRIGHT_VERSION=""

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl gnupg \
  && mkdir -p /etc/apt/keyrings \
  && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
  && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" > /etc/apt/sources.list.d/nodesource.list \
  && apt-get update \
  && apt-get install -y --no-install-recommends nodejs fontconfig ${APT_PACKAGES} \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /work/frontend

COPY package.json package-lock.json ./

ENV PLAYWRIGHT_BROWSERS_PATH=/ms-playwright

RUN echo "${PLAYWRIGHT_VERSION}" > /opt/e2e-playwright-version \
  && npm ci --no-audit --no-fund \
  && npx playwright install chromium \
  && chmod -R a+rwX /ms-playwright
