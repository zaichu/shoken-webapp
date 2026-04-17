use axum::http::Request as AxumRequest;
use tower_http::trace::MakeSpan;

/// TraceLayer 用スパンメーカー（クエリパラメータを除外）
///
/// `TraceLayer::new_for_http()` の既定実装はクエリ文字列込みの URI を記録するため、
/// OAuth コールバックの `code` / `state` など機密パラメータがログに残る。
/// このスパンメーカーはパスのみを記録し機密情報の漏洩を防ぐ。
#[derive(Clone, Copy, Debug)]
pub struct PathOnlyMakeSpan;

impl<B> MakeSpan<B> for PathOnlyMakeSpan {
    fn make_span(&mut self, request: &AxumRequest<B>) -> tracing::Span {
        // x-request-id は SetRequestIdLayer より内側で span を作るため、
        // ここで読んだ値がそのままログ相関 ID として機能する
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        tracing::debug_span!(
            "http_request",
            method = %request.method(),
            path = request.uri().path(),
            request_id = request_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use {
        super::PathOnlyMakeSpan,
        axum::http::Request,
        std::{
            collections::BTreeMap,
            sync::{Arc, Mutex},
        },
        tower_http::trace::MakeSpan,
        tracing::{
            field::{Field, Visit},
            Subscriber,
        },
        tracing_subscriber::{
            layer::{Context, SubscriberExt},
            registry::LookupSpan,
            Layer, Registry,
        },
    };

    #[derive(Clone, Debug, Default)]
    struct CapturedSpans(Arc<Mutex<Vec<CapturedSpan>>>);

    impl CapturedSpans {
        fn take(&self) -> Vec<CapturedSpan> {
            self.0
                .lock()
                .expect("captured spans mutex poisoned")
                .clone()
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedSpan {
        name: String,
        fields: BTreeMap<String, String>,
    }

    #[derive(Default)]
    struct FieldRecorder(BTreeMap<String, String>);

    impl Visit for FieldRecorder {
        fn record_str(&mut self, field: &Field, value: &str) {
            self.0.insert(field.name().to_string(), value.to_string());
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.0
                .insert(field.name().to_string(), format!("{value:?}"));
        }
    }

    impl<S> Layer<S> for CapturedSpans
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            _id: &tracing::Id,
            _ctx: Context<'_, S>,
        ) {
            let mut recorder = FieldRecorder::default();
            attrs.record(&mut recorder);
            self.0
                .lock()
                .expect("captured spans mutex poisoned")
                .push(CapturedSpan {
                    name: attrs.metadata().name().to_string(),
                    fields: recorder.0,
                });
        }
    }

    #[test]
    fn test_make_span_covers_request_id_extraction() {
        let captured = CapturedSpans::default();
        let subscriber = Registry::default().with(captured.clone());

        tracing::subscriber::with_default(subscriber, || {
            let mut make_span = PathOnlyMakeSpan;

            let request_with_id = Request::builder()
                .method("GET")
                .uri("/auth/google/callback?code=secret&state=opaque")
                .header("x-request-id", "req-123")
                .body(())
                .unwrap();
            let span = make_span.make_span(&request_with_id);
            drop(span);

            let request_without_id = Request::builder()
                .method("POST")
                .uri("/health?foo=bar")
                .body(())
                .unwrap();
            let span = make_span.make_span(&request_without_id);
            drop(span);
        });

        let spans = captured.take();
        assert_eq!(spans.len(), 2);

        assert_eq!(spans[0].name, "http_request");
        assert_eq!(
            spans[0].fields.get("request_id").map(String::as_str),
            Some("req-123")
        );
        assert_eq!(
            spans[0].fields.get("path").map(String::as_str),
            Some("/auth/google/callback")
        );
        assert_eq!(
            spans[0].fields.get("method").map(String::as_str),
            Some("GET")
        );

        assert_eq!(
            spans[1].fields.get("request_id").map(String::as_str),
            Some("-")
        );
        assert_eq!(
            spans[1].fields.get("path").map(String::as_str),
            Some("/health")
        );
        assert_eq!(
            spans[1].fields.get("method").map(String::as_str),
            Some("POST")
        );
    }
}
