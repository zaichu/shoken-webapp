# CLAUDE.md
日本語で必ず回答してください。
This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `npm run dev` - Start development server on port 8080 with auto-open
- `npm run build` - Production build to `dist/` directory
- `npm run lint` - Run ESLint with TypeScript and React rules
- `npm test` - Run Vitest tests
- `npm test:watch` - Run tests in watch mode

### Make Commands
- `make dev` - Run development server
- `make build` - Production build
- `make install` - Install dependencies
- `make clean` - Remove node_modules and dist
- `make all` - Clean, install, and build

## Architecture Overview

### Tech Stack
- **React 19.1.0** with TypeScript and Vite
- **TanStack React Query** for server state management
- **React Router DOM 7.6.0** for routing
- **Bootstrap 5.3.6** for styling
- **Vitest + Testing Library** for testing

### Project Structure
This is a Japanese stock/securities webapp following **Atomic Design** methodology:

- `src/components/atoms/` - Basic UI elements (Button, InputField, Table)
- `src/components/molecules/` - Composite components (CSVFileInput, ErrorBoundary)
- `src/components/organisms/` - Complex UI blocks (Header, SearchForm, ReceiptTable)
- `src/components/templates/` - Page layouts (Layout, ReceiptTemplate)
- `src/pages/` - Route-level components (Home, Search, Receipts, AssetBalance)
- `src/features/` - Domain logic (auth, jquants API, receipt processing, stock search)
- `src/lib/` - Core utilities (api client, csv processing, interfaces, utils)
- `src/hooks/` - Custom React hooks
- `src/contexts/` - React contexts

### Domain & Features
The app handles:
- **CSV Import**: Trading transaction data with encoding detection
- **Stock Search**: Japanese stock lookup by code/name
- **Receipt Management**: Trading analysis (domestic stock, mutual funds, dividends)
- **Tax Calculations**: Built-in Japanese tax rates (20.315%)
- **J-Quants API**: Financial data integration via `/api/jquants` proxy

### Key Architectural Patterns
- **Atomic Design**: Strict component hierarchy with single responsibility
- **Feature-based organization**: Domain logic grouped by business feature  
- **Custom HTTP client**: Axios wrapper with retry logic and error handling
- **Type-safe CSV processing**: Strong typing throughout import pipeline
- **Responsive tables**: Auto-resizing with grouping and summary rows

### Configuration Notes
- **Base path**: `/shoken-webapp/` for GitHub Pages deployment
- **Path aliases**: `@/*` maps to `src/*`
- **API proxy**: Development server proxies J-Quants API
- **Testing**: Single fork configuration with jsdom environment
- **Build**: Vendor chunk splitting with Terser minification

### Development Guidelines
- Components must follow Atomic Design categorization
- Business logic belongs in custom hooks or feature modules
- All CSV processing must maintain type safety
- Use React Query for server state management
- Japanese language throughout (comments and UI text)