#!/bin/bash
set -e

# pnpm を non-TTY 環境（CI・エージェント）でも動作させる
export CI=true

echo "📦 Bundling React app to single HTML artifact..."

# Check if we're in a project directory
if [ ! -f "package.json" ]; then
  echo "❌ Error: No package.json found. Run this script from your project root."
  exit 1
fi

# Check if index.html exists
if [ ! -f "index.html" ]; then
  echo "❌ Error: No index.html found in project root."
  echo "   This script requires an index.html entry point."
  exit 1
fi

# Install bundling dependencies（既にインストール済みならスキップ）
if [ ! -d "node_modules/parcel" ] || [ ! -d "node_modules/html-inline" ]; then
  echo "📦 Installing bundling dependencies..."
  # pnpm ストアの不整合（ERR_PNPM_UNEXPECTED_STORE）を解消してから追加する
  pnpm install
  pnpm add -D parcel @parcel/config-default parcel-resolver-tspaths html-inline
else
  echo "📦 Bundling dependencies already installed, skipping."
fi

# Create Parcel config with tspaths resolver
if [ ! -f ".parcelrc" ]; then
  echo "🔧 Creating Parcel configuration with path alias support..."
  cat > .parcelrc << 'EOF'
{
  "extends": "@parcel/config-default",
  "resolvers": ["parcel-resolver-tspaths", "..."]
}
EOF
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf dist bundle.html

# Parcel は /xxx をプロジェクトルート相対で解決するため、
# public/ 配下のファイルをルートに一時コピーし、終了時（成功・失敗問わず）に削除する
PUBLIC_COPIES=()
if [ -d "public" ]; then
  for src in public/*; do
    [ -e "$src" ] || continue
    dest=$(basename "$src")
    if [ ! -e "$dest" ]; then
      cp -r "$src" "$dest"
      PUBLIC_COPIES+=("$dest")
    fi
  done
fi

cleanup() {
  for f in "${PUBLIC_COPIES[@]+"${PUBLIC_COPIES[@]}"}"; do
    rm -rf "$f"
  done
}
trap cleanup EXIT

# Build with Parcel
echo "🔨 Building with Parcel..."
pnpm exec parcel build index.html --dist-dir dist --no-source-maps

# Inline everything into single HTML
echo "🎯 Inlining all assets into single HTML file..."
pnpm exec html-inline dist/index.html > bundle.html

# Get file size
FILE_SIZE=$(du -h bundle.html | cut -f1)

echo ""
echo "✅ Bundle complete!"
echo "📄 Output: bundle.html ($FILE_SIZE)"
echo ""
echo "You can now use this single HTML file as an artifact in Claude conversations."
echo "To test locally: open bundle.html in your browser"