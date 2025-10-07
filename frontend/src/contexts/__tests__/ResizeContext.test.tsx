import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ResizeProvider } from '../ResizeContext';
import { useForceResize, useTriggerResize } from '@/hooks/common/useResize';

// テスト用コンポーネント
const TestConsumer = () => {
  const forceResize = useForceResize();
  const triggerResize = useTriggerResize();

  return (
    <div>
      <div data-testid="force-resize-value">{forceResize}</div>
      <button onClick={() => triggerResize?.()} data-testid="trigger-button">
        Trigger Resize
      </button>
    </div>
  );
};

const TestConsumerWithoutProvider = () => {
  const forceResize = useForceResize();
  const triggerResize = useTriggerResize();

  return (
    <div>
      <div data-testid="force-resize-value">{forceResize ?? 'undefined'}</div>
      <button onClick={() => triggerResize?.()} data-testid="trigger-button">
        Trigger Resize
      </button>
    </div>
  );
};

describe('ResizeContext', () => {
  test('ResizeProviderが正しく動作する', () => {
    render(
      <ResizeProvider>
        <TestConsumer />
      </ResizeProvider>
    );

    // 初期値は0
    expect(screen.getByTestId('force-resize-value')).toHaveTextContent('0');

    // ボタンをクリックして値が更新されることを確認
    const triggerButton = screen.getByTestId('trigger-button');
    fireEvent.click(triggerButton);

    expect(screen.getByTestId('force-resize-value')).toHaveTextContent('1');

    // 再度クリックして値がインクリメントされることを確認
    fireEvent.click(triggerButton);
    expect(screen.getByTestId('force-resize-value')).toHaveTextContent('2');
  });

  test('複数回のtriggerResizeが正しく動作する', () => {
    render(
      <ResizeProvider>
        <TestConsumer />
      </ResizeProvider>
    );

    const triggerButton = screen.getByTestId('trigger-button');

    // 複数回クリック
    for (let i = 1; i <= 5; i++) {
      fireEvent.click(triggerButton);
      expect(screen.getByTestId('force-resize-value')).toHaveTextContent(String(i));
    }
  });

  test('Provider無しでhookを使用した場合undefinedが返される', () => {
    render(<TestConsumerWithoutProvider />);

    expect(screen.getByTestId('force-resize-value')).toHaveTextContent('undefined');

    // triggerResizeがundefinedでもエラーが発生しないことを確認
    const triggerButton = screen.getByTestId('trigger-button');
    expect(() => fireEvent.click(triggerButton)).not.toThrow();
  });

  test('複数のコンシューマーが同じ値を共有する', () => {
    const MultipleConsumers = () => (
      <ResizeProvider>
        <div data-testid="consumer1">
          <TestConsumer />
        </div>
        <div data-testid="consumer2">
          <TestConsumer />
        </div>
      </ResizeProvider>
    );

    render(<MultipleConsumers />);

    // 両方のコンシューマーが同じ初期値を持つ
    const forceResizeValues = screen.getAllByTestId('force-resize-value');
    expect(forceResizeValues[0]).toHaveTextContent('0');
    expect(forceResizeValues[1]).toHaveTextContent('0');

    // 一方のボタンをクリック
    const triggerButtons = screen.getAllByTestId('trigger-button');
    fireEvent.click(triggerButtons[0]);

    // 両方のコンシューマーが同じ更新された値を持つ
    expect(forceResizeValues[0]).toHaveTextContent('1');
    expect(forceResizeValues[1]).toHaveTextContent('1');
  });

  test('ネストしたProviderは独立して動作する', () => {
    const NestedProviders = () => (
      <ResizeProvider>
        <div data-testid="outer-consumer">
          <TestConsumer />
        </div>
        <ResizeProvider>
          <div data-testid="inner-consumer">
            <TestConsumer />
          </div>
        </ResizeProvider>
      </ResizeProvider>
    );

    render(<NestedProviders />);

    const outerConsumer = screen.getByTestId('outer-consumer');
    const innerConsumer = screen.getByTestId('inner-consumer');

    const outerForceResize = outerConsumer.querySelector('[data-testid="force-resize-value"]')!;
    const innerForceResize = innerConsumer.querySelector('[data-testid="force-resize-value"]')!;
    const outerTrigger = outerConsumer.querySelector('[data-testid="trigger-button"]')!;
    const innerTrigger = innerConsumer.querySelector('[data-testid="trigger-button"]')!;

    // 初期値は両方とも0
    expect(outerForceResize).toHaveTextContent('0');
    expect(innerForceResize).toHaveTextContent('0');

    // 外側のボタンをクリック
    fireEvent.click(outerTrigger);
    expect(outerForceResize).toHaveTextContent('1');
    expect(innerForceResize).toHaveTextContent('0'); // 内側は変更されない

    // 内側のボタンをクリック
    fireEvent.click(innerTrigger);
    expect(outerForceResize).toHaveTextContent('1'); // 外側は変更されない
    expect(innerForceResize).toHaveTextContent('1');
  });
});
