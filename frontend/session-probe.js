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
  // 時間切れ時は Wasm 側がリトライなしで1回だけ撃ち直すので、ここは1回分(5秒)に抑える。
  // 5s + 5s で従来経路の合計(AUTH_TIMEOUT 5s x 2回 + 間隔、約10.5秒)を超えない
  var timedOut = false;
  var timer = setTimeout(function () {
    timedOut = true;
    controller.abort();
  }, 5000);
  window.__shokenSessionProbe = fetch(base + '/api/v1/session', {
    credentials: 'include',
    signal: controller.signal,
  })
    .then(function (res) {
      if (res.status === 401) return { state: 'anonymous' };
      // HTTP の拒否(5xx 等)は確定した応答なので撃ち直しの対象にしない
      if (!res.ok) return { state: 'http_error' };
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
      // 通信失敗だけは Wasm 側のリトライ付き通常経路に切り替える
      return { state: timedOut ? 'timeout' : 'error' };
    })
    .finally(function () {
      clearTimeout(timer);
    });
})();
