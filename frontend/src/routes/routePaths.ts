export const ROUTE_PATHS = {
  home: '/',
  search: '/search',
  receipts: '/receipts',
  assetBalance: '/assetbalance',
  login: '/login',
  notFound: '/404',
} as const;

export type RoutePath = (typeof ROUTE_PATHS)[keyof typeof ROUTE_PATHS];
