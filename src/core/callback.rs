use std::future::Future;

use futures::channel::oneshot::{Receiver, Sender};
use parking_lot::Mutex;
use steamgear_sys as sys;
use thiserror::Error;

use crate::{client::SteamClient, utils::callbacks::SteamShutdown};

pub(crate) struct CallResult<T: CallbackTyped> {
    id: sys::SteamAPICall_t,
    client: SteamClient,
    _marker: std::marker::PhantomData<T>,
}

impl<T: CallbackTyped> CallResult<T> {
    pub(crate) fn new(id: sys::SteamAPICall_t, client: SteamClient) -> Self {
        Self {
            id,
            client,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: CallbackTyped> Future for CallResult<T> {
    type Output = Option<T>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(is_complete) = self.client.is_api_call_completed(self.id) {
            if is_complete {
                self.client.remove_call_result(self.id);
                std::task::Poll::Ready(self.client.get_api_call_result(self.id))
            } else {
                self.client
                    .register_call_result(self.id, cx.waker().clone());
                std::task::Poll::Pending
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }
}

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
    pub(crate) steam_shutdown_callback: OneshotDispatcher<SteamShutdown>,
}

pub(crate) trait CallbackDispatcher: Send + Sync {
    type Item: CallbackTyped;

    fn storage(&self) -> &OneshotDispatcher<Self::Item>;

    fn register(&self) -> Receiver<Self::Item> {
        let storage = &self.storage().inner;
        let mut guard = storage.lock();
        let (sender, receiver) = futures::channel::oneshot::channel();

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
            match sender.send(value) {
                Ok(_) => { /* TODO: Log all is okey*/ }
                Err(_) => {
                    // TODO: Storage is broken
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct OneshotDispatcher<T: CallbackTyped> {
    inner: Mutex<Option<Sender<T>>>,
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

    fn storage(&self) -> &OneshotDispatcher<Self::Item> {
        self
    }
}

#[derive(Clone, Error, Debug)]
pub enum CallbackError {
    #[error("This callback is pending elsewhere, the current request is canceled")]
    Canceled,
}
