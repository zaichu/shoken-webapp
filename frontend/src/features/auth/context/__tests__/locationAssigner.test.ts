import { afterEach, describe, expect, it, vi } from 'vitest';
import { locationAssigner } from '../locationAssigner';

describe('locationAssigner', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('assign が window.location.assign にURLを渡す', () => {
    const assign = vi.fn();

    vi.stubGlobal('window', {
      location: {
        assign,
      },
    });

    locationAssigner.assign('https://api.example.com/auth/google');

    expect(assign).toHaveBeenCalledWith('https://api.example.com/auth/google');
  });
});
