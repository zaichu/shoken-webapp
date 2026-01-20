# フロントエンドテストルール

## テストフレームワーク

- **Vitest**: テストランナー
- **React Testing Library**: コンポーネントテスト
- **jsdom**: DOM環境シミュレーション

## ディレクトリ構成

各コンポーネント・機能ごとに `__tests__/` ディレクトリを配置:

```
src/
├── components/
│   └── atoms/
│       └── Button/
│           ├── Button.tsx
│           └── __tests__/
│               └── Button.test.tsx
└── features/
    └── stock/
        └── __tests__/
            └── StockSearch.test.tsx
```

## テスト実行

```bash
npm test              # 全テスト実行
npm run test:watch    # ウォッチモード
npm run test:coverage # カバレッジ付き
```

## メモリ最適化

大規模テスト実行時のメモリ制限:

```bash
NODE_OPTIONS='--max-old-space-size=8192' npm test
```

## テストの書き方

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from '../Button';

describe('Button', () => {
  it('クリック時にonClickが呼ばれる', async () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>テスト</Button>);

    await userEvent.click(screen.getByRole('button'));

    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

## テスト原則

- ユーザー視点でテストを書く（実装詳細に依存しない）
- `getByRole`, `getByText` を優先使用
- `data-testid` は最終手段
- 非同期処理は `waitFor` または `findBy*` を使用
