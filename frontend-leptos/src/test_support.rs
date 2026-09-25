use std::future::Future;
use std::task::{Context, Poll, Waker};

/// 即座に完了するモック Future を同期的に駆動する。
pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    let mut context = Context::from_waker(Waker::noop());
    match std::pin::pin!(future).poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("モックの Future は即座に完了するはず"),
    }
}
