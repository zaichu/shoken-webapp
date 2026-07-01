import { describe, it, expect, vi, beforeEach } from 'vitest';
import { dividendApi, domesticStockApi, mutualfundApi } from '../receiptApi';
import { apiClient } from '@/lib/api/client';
import * as csvHelpers from '@/lib/api/csvHelpers';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
    delete: vi.fn(),
  },
}));

vi.mock('@/lib/api/csvHelpers', () => ({
  uploadCsvFile: vi.fn(),
  previewCsvFile: vi.fn(),
}));

describe('dividendApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list が /api/v1/dividends に GET リクエストを送る', () => {
    dividendApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/dividends', { withCredentials: true });
  });

  it('list が指定されたページング条件を送る', () => {
    dividendApi.list({ per_page: 1000 });
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/dividends', {
      params: { per_page: 1000 },
      withCredentials: true,
    });
  });

  it('previewCsv が /api/v1/dividend-import-validations にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    dividendApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/api/v1/dividend-import-validations', file);
  });

  it('uploadCsv が /api/v1/dividend-imports にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    dividendApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/api/v1/dividend-imports', file);
  });

  it('deleteAll が /api/v1/dividends に DELETE リクエストを送る', () => {
    dividendApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/dividends', { withCredentials: true });
  });
});

describe('domesticStockApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list が /api/v1/domestic-stock-transactions に GET リクエストを送る', () => {
    domesticStockApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/domestic-stock-transactions', { withCredentials: true });
  });

  it('list が指定されたページング条件を送る', () => {
    domesticStockApi.list({ per_page: 1000 });
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/domestic-stock-transactions', {
      params: { per_page: 1000 },
      withCredentials: true,
    });
  });

  it('previewCsv が /api/v1/domestic-stock-import-validations にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    domesticStockApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/api/v1/domestic-stock-import-validations', file);
  });

  it('uploadCsv が /api/v1/domestic-stock-imports にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    domesticStockApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/api/v1/domestic-stock-imports', file);
  });

  it('deleteAll が /api/v1/domestic-stock-transactions に DELETE リクエストを送る', () => {
    domesticStockApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/domestic-stock-transactions', { withCredentials: true });
  });
});

describe('mutualfundApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list が /api/v1/mutual-fund-transactions に GET リクエストを送る', () => {
    mutualfundApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/mutual-fund-transactions', { withCredentials: true });
  });

  it('list が指定されたページング条件を送る', () => {
    mutualfundApi.list({ per_page: 1000 });
    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/mutual-fund-transactions', {
      params: { per_page: 1000 },
      withCredentials: true,
    });
  });

  it('previewCsv が /api/v1/mutual-fund-import-validations にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    mutualfundApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/api/v1/mutual-fund-import-validations', file);
  });

  it('uploadCsv が /api/v1/mutual-fund-imports にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    mutualfundApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/api/v1/mutual-fund-imports', file);
  });

  it('deleteAll が /api/v1/mutual-fund-transactions に DELETE リクエストを送る', () => {
    mutualfundApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/mutual-fund-transactions', { withCredentials: true });
  });
});
