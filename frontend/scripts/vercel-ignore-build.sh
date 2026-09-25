#!/usr/bin/env bash
# デプロイは GitHub Actions の `vercel deploy --prebuilt` 経路のみ。
# Vercel の Git 連携ビルドは常にスキップして二重デプロイを防ぐ。
echo "Skipped: deployments are done via 'vercel deploy --prebuilt' from GitHub Actions."
exit 0
