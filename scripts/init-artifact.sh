#!/bin/bash

# Exit on error
set -e

# Detect Node version（vite@8 の engines.node: ^20.19.0 || >=22.12.0 に合わせて検証）
NODE_FULL=$(node -v | cut -d'v' -f2)
NODE_MAJOR=$(echo "$NODE_FULL" | cut -d'.' -f1)
NODE_MINOR=$(echo "$NODE_FULL" | cut -d'.' -f2)

echo "🔍 Detected Node.js version: $NODE_FULL"

NODE_OK=0
if [ "$NODE_MAJOR" -eq 20 ] && [ "$NODE_MINOR" -ge 19 ]; then
  NODE_OK=1
elif [ "$NODE_MAJOR" -eq 22 ] && [ "$NODE_MINOR" -ge 12 ]; then
  NODE_OK=1
elif [ "$NODE_MAJOR" -gt 22 ]; then
  NODE_OK=1
fi

if [ "$NODE_OK" -eq 0 ]; then
  echo "❌ Error: Node.js ^20.19.0 || >=22.12.0 is required"
  echo "   Current version: $NODE_FULL"
  echo "   Note: vite@8 and @vitejs/plugin-react@6 require Node ^20.19.0 || >=22.12.0"
  exit 1
fi

echo "✅ Node.js $NODE_FULL: OK"

# Detect OS and set sed syntax（配列で保持することで '' が literal にならない）
if [[ "$OSTYPE" == "darwin"* ]]; then
  SED_INPLACE=("sed" "-i" "")
else
  SED_INPLACE=("sed" "-i")
fi

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
  if command -v corepack &> /dev/null; then
    echo "📦 pnpm not found. Installing via corepack..."
    corepack enable pnpm
  else
    echo "❌ Error: pnpm is required but not installed."
    echo "   Install with: corepack enable pnpm"
    echo "   Or: npm install -g pnpm"
    exit 1
  fi
fi

# Check if project name is provided
if [ -z "$1" ]; then
  echo "❌ Usage: ./init-artifact.sh <project-name>"
  exit 1
fi

PROJECT_NAME="$1"

# プロジェクト名にパス区切り文字が含まれる場合は拒否（sed やパス操作が壊れるため）
if [[ "$PROJECT_NAME" =~ [/\\] ]]; then
  echo "❌ Error: project name must be a simple name without path separators."
  echo "   Use: ./init-artifact.sh my-project"
  exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPONENTS_TARBALL="$SCRIPT_DIR/shadcn-components.tar.gz"

# Check if components tarball exists
if [ ! -f "$COMPONENTS_TARBALL" ]; then
  echo "❌ Error: shadcn-components.tar.gz not found in script directory"
  echo "   Expected location: $COMPONENTS_TARBALL"
  exit 1
fi

echo "🚀 Creating new React + Vite project: $PROJECT_NAME"

# create-vite で React+TypeScript テンプレートを生成する
# create-vite のバージョンと vite テンプレートのバージョンは一致しないため、
# 生成後に vite / @vitejs/plugin-react を明示的に shoken-artifact と同じバージョンに上書きする
echo "" | pnpm create vite "$PROJECT_NAME" --template react-ts

# Navigate into project directory
cd "$PROJECT_NAME"

echo "🧹 Cleaning up Vite template..."
"${SED_INPLACE[@]}" '/<link rel="icon".*vite\.svg/d' index.html
"${SED_INPLACE[@]}" 's/<title>.*<\/title>/<title>'"$PROJECT_NAME"'<\/title>/' index.html
# Vite テンプレの starter ファイルを削除（artifact 基盤には不要）
rm -f src/App.tsx src/App.css
rm -rf src/assets

echo "📦 Installing base dependencies (vite@8 + React 19 を明示固定)..."
# create-vite テンプレートのバージョンに依存しないよう、shoken-artifact と同じバージョンで上書き
pnpm install vite@^8.0.0 "@vitejs/plugin-react@^6.0.0" react@^19.2.4 react-dom@^19.2.4 typescript@~5.9.3

echo "📦 Installing Tailwind CSS and dependencies..."
pnpm install -D tailwindcss@3.4.1 postcss autoprefixer @types/node tailwindcss-animate
pnpm install class-variance-authority clsx tailwind-merge lucide-react next-themes

echo "⚙️  Creating Tailwind and PostCSS configuration..."
cat > postcss.config.js << 'EOF'
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOF

echo "📝 Configuring Tailwind with shadcn theme..."
cat > tailwind.config.js << 'EOF'
/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
EOF

# Add Tailwind directives and CSS variables to index.css
echo "🎨 Adding Tailwind directives and CSS variables..."
cat > src/index.css << 'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 0 0% 3.9%;
    --card: 0 0% 100%;
    --card-foreground: 0 0% 3.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 0 0% 3.9%;
    --primary: 0 0% 9%;
    --primary-foreground: 0 0% 98%;
    --secondary: 0 0% 96.1%;
    --secondary-foreground: 0 0% 9%;
    --muted: 0 0% 96.1%;
    --muted-foreground: 0 0% 45.1%;
    --accent: 0 0% 96.1%;
    --accent-foreground: 0 0% 9%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 0 0% 98%;
    --border: 0 0% 89.8%;
    --input: 0 0% 89.8%;
    --ring: 0 0% 3.9%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 0 0% 3.9%;
    --foreground: 0 0% 98%;
    --card: 0 0% 3.9%;
    --card-foreground: 0 0% 98%;
    --popover: 0 0% 3.9%;
    --popover-foreground: 0 0% 98%;
    --primary: 0 0% 98%;
    --primary-foreground: 0 0% 9%;
    --secondary: 0 0% 14.9%;
    --secondary-foreground: 0 0% 98%;
    --muted: 0 0% 14.9%;
    --muted-foreground: 0 0% 63.9%;
    --accent: 0 0% 14.9%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 0 0% 98%;
    --border: 0 0% 14.9%;
    --input: 0 0% 14.9%;
    --ring: 0 0% 83.1%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
EOF

# Add path aliases to tsconfig.json
echo "🔧 Adding path aliases to tsconfig.json..."
node -e "
const fs = require('fs');
const config = JSON.parse(fs.readFileSync('tsconfig.json', 'utf8'));
config.compilerOptions = config.compilerOptions || {};
config.compilerOptions.baseUrl = '.';
config.compilerOptions.paths = { '@/*': ['./src/*'] };
fs.writeFileSync('tsconfig.json', JSON.stringify(config, null, 2));
"

# Add path aliases to tsconfig.app.json
echo "🔧 Adding path aliases to tsconfig.app.json..."
node -e "
const fs = require('fs');
const path = 'tsconfig.app.json';
const content = fs.readFileSync(path, 'utf8');
// Remove comments manually
const lines = content.split('\n').filter(line => !line.trim().startsWith('//'));
const jsonContent = lines.join('\n');
const config = JSON.parse(jsonContent.replace(/\/\*[\s\S]*?\*\//g, '').replace(/,(\s*[}\]])/g, '\$1'));
config.compilerOptions = config.compilerOptions || {};
config.compilerOptions.baseUrl = '.';
config.compilerOptions.paths = { '@/*': ['./src/*'] };
fs.writeFileSync(path, JSON.stringify(config, null, 2));
"

# Update vite.config.ts
echo "⚙️  Updating Vite configuration..."
cat > vite.config.ts << 'EOF'
import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
EOF

# Install all shadcn/ui dependencies
echo "📦 Installing shadcn/ui dependencies..."
pnpm install @radix-ui/react-accordion @radix-ui/react-aspect-ratio @radix-ui/react-avatar @radix-ui/react-checkbox @radix-ui/react-collapsible @radix-ui/react-context-menu @radix-ui/react-dialog @radix-ui/react-dropdown-menu @radix-ui/react-hover-card @radix-ui/react-label @radix-ui/react-menubar @radix-ui/react-navigation-menu @radix-ui/react-popover @radix-ui/react-progress @radix-ui/react-radio-group @radix-ui/react-scroll-area @radix-ui/react-select @radix-ui/react-separator @radix-ui/react-slider @radix-ui/react-slot @radix-ui/react-switch @radix-ui/react-tabs @radix-ui/react-toast @radix-ui/react-toggle @radix-ui/react-toggle-group @radix-ui/react-tooltip
pnpm install "sonner@^2.0.7" "cmdk@^1.1.1" "vaul@^1.1.2" "embla-carousel-react@^8.6.0" "react-day-picker@^9.14.0" "date-fns@^4.1.0" "react-hook-form@^7.71.2" "@hookform/resolvers@^5.2.2" "zod@^4.3.6"

# Extract shadcn components from tarball
echo "📦 Extracting shadcn/ui components..."
tar -xzf "$COMPONENTS_TARBALL" -C src/

# Update eslint.config.js to disable react-refresh rules for shadcn/ui files
echo "🔧 Updating ESLint config for shadcn/ui components..."
cat > eslint.config.js << 'EOF'
import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
  },
  {
    // shadcn/ui コンポーネントは CVA 等の非コンポーネント値も export するため
    // react-refresh のルールを除外する
    files: ['src/components/ui/**/*.{ts,tsx}', 'src/hooks/use-toast.ts'],
    rules: {
      'react-refresh/only-export-components': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
    },
  },
])
EOF

# Create components.json for reference
echo "📝 Creating components.json config..."
cat > components.json << 'EOF'
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "default",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "tailwind.config.js",
    "css": "src/index.css",
    "baseColor": "slate",
    "cssVariables": true,
    "prefix": ""
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "hooks": "@/hooks"
  }
}
EOF

echo "✅ Setup complete! You can now use Tailwind CSS and shadcn/ui in your project."
echo ""
echo "📦 Included components (40+ total):"
echo "  - accordion, alert, aspect-ratio, avatar, badge, breadcrumb"
echo "  - button, calendar, card, carousel, checkbox, collapsible"
echo "  - command, context-menu, dialog, drawer, dropdown-menu"
echo "  - form, hover-card, input, label, menubar, navigation-menu"
echo "  - popover, progress, radio-group, scroll-area"
echo "  - select, separator, sheet, skeleton, slider, sonner"
echo "  - switch, table, tabs, textarea, toast, toggle, toggle-group, tooltip"
echo ""
echo "To start developing:"
echo "  cd $PROJECT_NAME"
echo "  pnpm dev"
echo ""
echo "📚 Import components like:"
echo "  import { Button } from '@/components/ui/button'"
echo "  import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'"
echo "  import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog'"
