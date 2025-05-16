use std::time::{Duration, Instant};

pub struct Toast {
    pub message: String,
    created_at: Instant,
    duration: Duration,
    pub variant: ToastVariant
}

pub enum ToastVariant {
    Error,
    Success
}

impl Toast {
    pub fn new<S: Into<String>>(message: S, r#type: ToastVariant) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            variant: r#type
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}
