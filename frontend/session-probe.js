// Wasm の読み込み・初期化と並行してセッション確認を出す起動時プローブ。
// Wasm 側は window.__shokenSessionProbe の結果を受け取り、同じリクエストを撃ち直さない。
// CSP の script-src 'self' を守るためインラインでは書かず、外部ファイルで配る。
(function () {
  // ログアウトが未送信で保留中なら、消えるはずのセッションを読みに行かない
  // (Wasm 側が先にログアウトを再送する。pending_logout.rs のキーと同一にすること)
  try {
    if (localStorage.getItem('pending_logout')) return;
  } catch (_) {
    // localStorage が使えない環境では判定を飛ばしてプローブする
  }
  var meta = document.querySelector('meta[name="shoken-api-origin"]');
  // ローカル開発では content が空 = 同一オリジン /api(trunk のプロキシが受ける)
  var base = meta && meta.content ? meta.content : '';
  var controller = new AbortController();
  // 従来経路の1回目分として待つ。時間切れなら Wasm 側が残り予算で撃ち直す(合計は約10.5秒)
  var timedOut = false;
  var timer = setTimeout(function () {
    timedOut = true;
    controller.abort();
  }, 3000);
  window.__shokenSessionProbe = fetch(base + '/api/v1/session', {
    credentials: 'include',
    signal: controller.signal,
  })
    .then(function (res) {
      if (res.status === 401) return { state: 'anonymous' };
      // 5xx は Wasm 側が従来経路と同じく撃ち直す。それ以外の 4xx は確定失敗
      if (!res.ok) return { state: res.status >= 500 ? 'http_error' : 'anonymous' };
      return res.json().then(
        function (user) {
          return { state: 'authenticated', user: user };
        },
        function () {
          return { state: 'error' };
        }
      );
    })
    .catch(function () {
      // 通信失敗・時間切れも Wasm 側の通常経路で撃ち直す
      return { state: timedOut ? 'timeout' : 'error' };
    })
    .finally(function () {
      clearTimeout(timer);
    });
})();
