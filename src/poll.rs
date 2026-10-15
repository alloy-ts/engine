use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Creates a future that wraps a function returning [`Poll`].
///
/// Polling the future delegates to the wrapped function. If the returned future is
/// pinned, then the captured environment of the wrapped function is also pinned in-place,
/// so as long as the closure does not move out of its captures it can soundly create pinned
/// references to them.
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
  F: FnMut(&mut Context<'_>) -> Poll<T>,
{
  PollFn { f }
}

/// A Future that wraps a function returning [`Poll`].
///
/// This `struct` is created by [`poll_fn()`]. See its documentation for more.
pub struct PollFn<F> {
  f: F,
}

impl<F> fmt::Debug for PollFn<F> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("PollFn").finish_non_exhaustive()
  }
}

impl<T, F> Future for PollFn<F>
where
  F: FnMut(&mut Context<'_>) -> Poll<T>,
{
  type Output = T;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
    // SAFETY: We do not move out of `self.f`.
    let this = unsafe { self.get_unchecked_mut() };
    (this.f)(cx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::task::{RawWaker, RawWakerVTable, Waker};

  fn dummy_waker() -> Waker {
    static VTABLE: RawWakerVTable = RawWakerVTable::new(
      |_| RawWaker::new(std::ptr::null(), &VTABLE),
      |_| {},
      |_| {},
      |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
  }

  #[test]
  fn test_poll_fn() {
    let mut count = 0;
    let mut f = poll_fn(move |_cx| {
      count += 1;
      if count == 2 {
        Poll::Ready("Hello, World!".to_string())
      } else {
        Poll::Pending
      }
    });

    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);

    let mut pin_f = Pin::new(&mut f);
    assert_eq!(pin_f.as_mut().poll(&mut cx), Poll::Pending);
    assert_eq!(
      pin_f.as_mut().poll(&mut cx),
      Poll::Ready("Hello, World!".to_string())
    );
  }
}
