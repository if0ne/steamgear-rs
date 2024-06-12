use futures::channel::{mpsc, oneshot};
use parking_lot::Mutex;
use thiserror::Error;

use crate::{apps::callbacks::DlcInstalled, utils::callbacks::SteamShutdown};

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
    pub(crate) dlc_installed_callback: OneshotDispatcher<DlcInstalled>,
}

pub(crate) trait CallbackDispatcher: Send + Sync {
    type Item: CallbackTyped;
    type Output;

    fn register(&self) -> Self::Output;
    fn proceed(&self, value: Self::Item);
}

#[derive(Debug)]
pub(crate) struct SingleDispatcher<T: CallbackTyped> {
    inner: Mutex<Option<mpsc::UnboundedSender<T>>>,
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
    type Output = mpsc::UnboundedReceiver<Self::Item>;

    fn register(&self) -> Self::Output {
        let storage = &self.inner;
        let mut guard = storage.lock();
        let (sender, receiver) = futures::channel::mpsc::unbounded();

        if guard.replace(sender).is_some() {
            tracing::warn!(
                "Callback {} have already registered, old request will be cancelled",
                std::any::type_name::<Self>()
            )
        }

        receiver
    }

    fn proceed(&self, value: Self::Item) {
        let storage = &self.inner;
        let guard = storage.lock();

        if let Some(sender) = &*guard {
            match sender.unbounded_send(value) {
                Ok(_) => {
                    tracing::debug!("Sent callback: {}", std::any::type_name::<Self>())
                }
                Err(_) => {
                    tracing::error!(
                        "Callback {} have received, but receiver is broken",
                        std::any::type_name::<Self>()
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct OneshotDispatcher<T: CallbackTyped> {
    inner: Mutex<Option<oneshot::Sender<T>>>,
}

impl<T: CallbackTyped> Default for OneshotDispatcher<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: CallbackTyped> CallbackDispatcher for OneshotDispatcher<T> {
    type Item = T;
    type Output = oneshot::Receiver<Self::Item>;

    fn register(&self) -> Self::Output {
        let storage = &self.inner;
        let mut guard = storage.lock();
        let (sender, receiver) = futures::channel::oneshot::channel();

        if guard.replace(sender).is_some() {
            tracing::warn!(
                "Callback {} have already registered, old request will be cancelled",
                std::any::type_name::<Self>()
            )
        }

        receiver
    }

    fn proceed(&self, value: Self::Item) {
        let storage = &self.inner;
        let mut guard = storage.lock();

        if let Some(sender) = guard.take() {
            match sender.send(value) {
                Ok(_) => {
                    tracing::debug!("Sent callback: {}", std::any::type_name::<Self>())
                }
                Err(_) => {
                    tracing::error!(
                        "Callback {} have received, but receiver is broken",
                        std::any::type_name::<Self>()
                    )
                }
            }
        }
    }
}

#[derive(Clone, Error, Debug)]
pub enum CallbackError {
    #[error("This callback is pending elsewhere, the current request is canceled")]
    Canceled,
}
