use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use parking_lot::Mutex;
use thiserror::Error;

use crate::utils::callbacks::SteamShutdown;

pub(crate) trait CallbackTyped: Clone + Send + 'static {
    const TYPE: u32;
    type Raw: Copy;

    fn from_raw(raw: Self::Raw) -> Self;

    unsafe fn from_ptr(ptr: *mut u8) -> Self::Raw {
        assert_eq!(
            std::mem::align_of::<Self::Raw>(),
            std::mem::align_of_val(&ptr)
        );

        let raw_type: Self::Raw = *(ptr as *const Self::Raw);

        raw_type
    }
}

#[derive(Debug, Default)]
pub(crate) struct CallbackContainer {
    pub(crate) steam_shutdown_callback: SingleDispatcher<SteamShutdown>,
}

pub(crate) trait CallbackDispatcher: Send + Sync {
    type Item: CallbackTyped;

    fn storage(&self) -> &SingleDispatcher<Self::Item>;

    fn register(&self) -> UnboundedReceiver<Self::Item> {
        let storage = &self.storage().inner;
        let mut guard = storage.lock();
        let (sender, receiver) = futures::channel::mpsc::unbounded();

        if guard.replace(sender).is_some() {
            // TODO: Log it was already registered
        }

        receiver
    }

    fn proceed(&self, value: Self::Item) {
        let storage = &self.storage().inner;
        let mut guard = storage.lock();

        let sender = guard.take();

        if let Some(sender) = sender {
            match sender.unbounded_send(value) {
                Ok(_) => { /* TODO: Log all is okey*/ }
                Err(_) => {
                    // TODO: Storage is broken
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct SingleDispatcher<T: CallbackTyped> {
    inner: Mutex<Option<UnboundedSender<T>>>,
}

impl<T: CallbackTyped> Default for SingleDispatcher<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: CallbackTyped> CallbackDispatcher for SingleDispatcher<T> {
    type Item = T;

    fn storage(&self) -> &SingleDispatcher<Self::Item> {
        self
    }
}

#[derive(Clone, Error, Debug)]
pub enum CallbackError {
    #[error("This callback is pending elsewhere, the current request is canceled")]
    Canceled,
}
