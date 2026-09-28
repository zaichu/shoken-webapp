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
  // Wasm 側の認証クライアントより先に出る分、DB の起動待ちを覆う長めの上限を取る
  var timer = setTimeout(function () {
    controller.abort();
  }, 15000);
  window.__shokenSessionProbe = fetch(base + '/api/v1/session', {
    credentials: 'include',
    signal: controller.signal,
  })
    .then(function (res) {
      if (res.status === 401) return { state: 'anonymous' };
      if (!res.ok) return { state: 'error' };
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
      // ネットワーク系の失敗は Wasm 側でリトライ付きの通常経路に切り替える
      return { state: 'error' };
    })
    .finally(function () {
      clearTimeout(timer);
    });
})();
