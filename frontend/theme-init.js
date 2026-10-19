// wasm の起動前にテーマを確定させる。ダーク設定の利用者に初期描画が白く点滅するのを防ぐ。
// 値が不正なときは既定の「端末に合わせる」と同じ扱いにする(settings.rs の Theme と一致させる)
(function () {
  var theme = null;
  try {
    theme = localStorage.getItem('setting_theme');
  } catch (error) {}
  var dark =
    theme === 'dark' ||
    ((!theme || theme === 'system') &&
      window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
})();
