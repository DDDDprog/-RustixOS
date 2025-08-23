use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub mod simple_executor;
pub mod keyboard;
pub mod executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub struct Waker {
    task_id: TaskId,
}

impl Waker {
    fn new(task_id: TaskId) -> Waker {
        Waker { task_id }
    }

    fn wake_task(&self) {
        // In a real implementation, this would notify the executor
        // that the task with the given ID should be polled again
    }
}

use core::task::Wake;

impl Wake for Waker {
    fn wake(self: alloc::sync::Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &alloc::sync::Arc<Self>) {
        self.wake_task();
    }
}