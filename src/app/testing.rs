use std::sync::{Arc, Mutex};

use super::{AppProxyError, AppProxyErrorCode, WakeBridge};

#[derive(Clone, Debug, Default)]
pub struct FakeWakeBridge {
    state: Arc<Mutex<FakeWakeState>>,
}

#[derive(Clone, Debug, Default)]
struct FakeWakeState {
    closed: bool,
    wakes: usize,
}

impl FakeWakeBridge {
    #[must_use]
    pub fn closed() -> Self {
        let bridge = Self::default();
        bridge.state.lock().expect("fake wake bridge lock").closed = true;
        bridge
    }

    #[must_use]
    pub fn wake_count(&self) -> usize {
        self.state.lock().expect("fake wake bridge lock").wakes
    }
}

impl WakeBridge for FakeWakeBridge {
    fn wake(&self) -> Result<(), AppProxyError> {
        let mut state = self.state.lock().expect("fake wake bridge lock");
        if state.closed {
            return Err(AppProxyError::new(AppProxyErrorCode::WakeFailed));
        }
        state.wakes += 1;
        Ok(())
    }
}
