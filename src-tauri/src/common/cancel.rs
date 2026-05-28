use std::sync::atomic::{AtomicBool, Ordering};

/// 跨组件共享的取消信号，使用原子布尔保证线程安全。
pub struct CancelToken(AtomicBool);

impl CancelToken {
    pub fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
