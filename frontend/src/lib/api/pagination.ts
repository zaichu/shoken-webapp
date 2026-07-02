interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

/**
 * per_page 上限（最大1000）を超えるレスポンスを、total に達するまで
 * ページを繰り上げて取得し、全件を1つの配列に結合する
 */
export async function fetchAllPages<T>(
  fetchPage: (page: number) => Promise<PaginatedResponse<T>>
): Promise<T[]> {
  const firstPage = await fetchPage(1);
  const rows = [...firstPage.data];

  const totalPages = Math.ceil(firstPage.total / firstPage.per_page);
  for (let page = firstPage.page + 1; page <= totalPages; page += 1) {
    const nextPage = await fetchPage(page);
    if (nextPage.data.length === 0) break;
    rows.push(...nextPage.data);
  }

  return rows;
}
