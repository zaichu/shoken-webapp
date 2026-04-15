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

  it('list が /dividends に GET リクエストを送る', () => {
    dividendApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/dividends', { withCredentials: true });
  });

  it('previewCsv が /dividends/csv/preview にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    dividendApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/dividends/csv/preview', file);
  });

  it('uploadCsv が /dividends/csv にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    dividendApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/dividends/csv', file);
  });

  it('deleteAll が /dividends に DELETE リクエストを送る', () => {
    dividendApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/dividends', { withCredentials: true });
  });
});

describe('domesticStockApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list が /domestic-stocks に GET リクエストを送る', () => {
    domesticStockApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/domestic-stocks', { withCredentials: true });
  });

  it('previewCsv が /domestic-stocks/csv/preview にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    domesticStockApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/domestic-stocks/csv/preview', file);
  });

  it('uploadCsv が /domestic-stocks/csv にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    domesticStockApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/domestic-stocks/csv', file);
  });

  it('deleteAll が /domestic-stocks に DELETE リクエストを送る', () => {
    domesticStockApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/domestic-stocks', { withCredentials: true });
  });
});

describe('mutualfundApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list が /mutualfunds に GET リクエストを送る', () => {
    mutualfundApi.list();
    expect(apiClient.get).toHaveBeenCalledWith('/mutualfunds', { withCredentials: true });
  });

  it('previewCsv が /mutualfunds/csv/preview にプレビューリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    mutualfundApi.previewCsv(file);
    expect(csvHelpers.previewCsvFile).toHaveBeenCalledWith('/mutualfunds/csv/preview', file);
  });

  it('uploadCsv が /mutualfunds/csv にアップロードリクエストを送る', () => {
    const file = new File(['content'], 'test.csv', { type: 'text/csv' });
    mutualfundApi.uploadCsv(file);
    expect(csvHelpers.uploadCsvFile).toHaveBeenCalledWith('/mutualfunds/csv', file);
  });

  it('deleteAll が /mutualfunds に DELETE リクエストを送る', () => {
    mutualfundApi.deleteAll();
    expect(apiClient.delete).toHaveBeenCalledWith('/mutualfunds', { withCredentials: true });
  });
});
