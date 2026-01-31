/**
 * クラス名結合ユーティリティ
 * - falsy な値（false, null, undefined, ''）を除外
 * - 複数のクラス名を空白で結合
 *
 * @example
 * cn('base', isActive && 'active', className)
 * // isActive=true, className='custom' => 'base active custom'
 */
export function cn(
  ...classes: (string | boolean | undefined | null)[]
): string {
  return classes.filter(Boolean).join(' ');
}
