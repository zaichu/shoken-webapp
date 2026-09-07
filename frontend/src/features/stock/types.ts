import type { components } from '@/generated/api';

// 生成型のエイリアス（API契約の正本は docs/openapi.json）
export type StockData = components['schemas']['Stock'];
