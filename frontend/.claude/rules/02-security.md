# フロントエンドセキュリティルール

## 環境変数

```bash
# .env.local (gitignore)
VITE_API_KEY=xxx
```

- `VITE_` プレフィックスが必要
- 機密情報はバックエンド経由で取得

## 入力バリデーション

```typescript
// 必ずサーバーサイドでも検証
const validateStockCode = (code: string): boolean => {
  return /^[0-9]{4}$/.test(code);
};
```

## XSS対策

### React の自動エスケープ

```tsx
// 安全（自動エスケープ）
<div>{userInput}</div>

// 危険 - 必要な場合のみ、サニタイズ後に使用
<div dangerouslySetInnerHTML={{ __html: sanitizedHtml }} />
```

## 依存関係のセキュリティ

```bash
npm audit              # 脆弱性チェック
npm audit fix          # 自動修正
npm update             # 依存関係更新
```
