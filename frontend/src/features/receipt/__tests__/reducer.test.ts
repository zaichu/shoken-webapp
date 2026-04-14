import { describe, expect, it } from 'vitest';

import {
  initialState,
  receiptsReducer,
  type CsvPreview,
  type ImportResult,
  type ReceiptsState,
} from '../reducer';

function makeCsvPreview(totalRows: number): CsvPreview {
  return {
    totalRows,
    validRows: totalRows - 1,
    errors: [{ row: totalRows, message: `${totalRows}行目にエラーがあります` }],
    rows: [{ id: totalRows }],
  };
}

function makeImportResult(inserted: number): ImportResult {
  return {
    inserted,
    skipped: 1,
    errors: [{ row: inserted + 1, message: `${inserted + 1}行目をスキップしました` }],
  };
}

function makeState(): ReceiptsState {
  return {
    receiptsType: 'domesticstock',
    rawFiles: {
      dividend: new File(['dividend'], 'dividend.csv', { type: 'text/csv' }),
      domesticstock: new File(['domestic'], 'domesticstock.csv', { type: 'text/csv' }),
      mutualfund: new File(['mutualfund'], 'mutualfund.csv', { type: 'text/csv' }),
    },
    csvPreviews: {
      dividend: makeCsvPreview(3),
      domesticstock: makeCsvPreview(4),
      mutualfund: makeCsvPreview(5),
    },
    lastImportResults: {
      dividend: makeImportResult(1),
      domesticstock: makeImportResult(2),
      mutualfund: makeImportResult(3),
    },
    showDeleteConfirm: true,
  };
}

describe('receiptsReducer', () => {
  it('SET_RECEIPTS_TYPE: receiptsType を更新する', () => {
    const next = receiptsReducer(initialState, {
      type: 'SET_RECEIPTS_TYPE',
      payload: 'mutualfund',
    });

    expect(next.receiptsType).toBe('mutualfund');
    expect(next.rawFiles).toEqual(initialState.rawFiles);
    expect(next.showDeleteConfirm).toBe(initialState.showDeleteConfirm);
  });

  it('SET_RAW_FILE: rawFiles を更新し、対象タブの preview と import result をリセットする', () => {
    const state = makeState();
    const nextFile = new File(['updated'], 'updated-mutualfund.csv', { type: 'text/csv' });

    const next = receiptsReducer(state, {
      type: 'SET_RAW_FILE',
      receiptsType: 'mutualfund',
      payload: nextFile,
    });

    expect(next.rawFiles.mutualfund).toBe(nextFile);
    expect(next.csvPreviews.mutualfund).toBeNull();
    expect(next.lastImportResults.mutualfund).toBeNull();
    expect(next.rawFiles.dividend).toBe(state.rawFiles.dividend);
    expect(next.rawFiles.domesticstock).toBe(state.rawFiles.domesticstock);
    expect(next.csvPreviews.dividend).toBe(state.csvPreviews.dividend);
    expect(next.csvPreviews.domesticstock).toBe(state.csvPreviews.domesticstock);
    expect(next.lastImportResults.dividend).toBe(state.lastImportResults.dividend);
    expect(next.lastImportResults.domesticstock).toBe(state.lastImportResults.domesticstock);
  });

  it('SET_CSV_PREVIEW: csvPreviews を更新する', () => {
    const state = makeState();
    const preview = makeCsvPreview(10);

    const next = receiptsReducer(state, {
      type: 'SET_CSV_PREVIEW',
      receiptsType: 'dividend',
      payload: preview,
    });

    expect(next.csvPreviews.dividend).toEqual(preview);
    expect(next.csvPreviews.domesticstock).toBe(state.csvPreviews.domesticstock);
    expect(next.csvPreviews.mutualfund).toBe(state.csvPreviews.mutualfund);
  });

  it('SET_IMPORT_RESULT: lastImportResults を更新する', () => {
    const state = makeState();
    const result = makeImportResult(10);

    const next = receiptsReducer(state, {
      type: 'SET_IMPORT_RESULT',
      receiptsType: 'domesticstock',
      payload: result,
    });

    expect(next.lastImportResults.domesticstock).toEqual(result);
    expect(next.lastImportResults.dividend).toBe(state.lastImportResults.dividend);
    expect(next.lastImportResults.mutualfund).toBe(state.lastImportResults.mutualfund);
  });

  it('CLEAR_IMPORT_RESULT: 対象タブの importResult を null にする', () => {
    const state = makeState();

    const next = receiptsReducer(state, {
      type: 'CLEAR_IMPORT_RESULT',
      receiptsType: 'dividend',
    });

    expect(next.lastImportResults.dividend).toBeNull();
    expect(next.lastImportResults.domesticstock).toBe(state.lastImportResults.domesticstock);
    expect(next.lastImportResults.mutualfund).toBe(state.lastImportResults.mutualfund);
  });

  it('SET_SHOW_DELETE_CONFIRM: showDeleteConfirm を更新する', () => {
    const next = receiptsReducer(initialState, {
      type: 'SET_SHOW_DELETE_CONFIRM',
      payload: true,
    });

    expect(next.showDeleteConfirm).toBe(true);
    expect(next.receiptsType).toBe(initialState.receiptsType);
  });

  it('LOGOUT: タブごとのファイル・preview・import result を全てリセットし、他の状態は維持する', () => {
    const state = makeState();

    const next = receiptsReducer(state, { type: 'LOGOUT' });

    expect(next.rawFiles).toEqual({
      dividend: null,
      domesticstock: null,
      mutualfund: null,
    });
    expect(next.csvPreviews).toEqual({
      dividend: null,
      domesticstock: null,
      mutualfund: null,
    });
    expect(next.lastImportResults).toEqual({
      dividend: null,
      domesticstock: null,
      mutualfund: null,
    });
    expect(next.receiptsType).toBe(state.receiptsType);
    expect(next.showDeleteConfirm).toBe(state.showDeleteConfirm);
  });
});
