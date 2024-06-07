use std::future::Future;

use parking_lot::Mutex;
use steamgear_sys as sys;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;

use crate::{client::SteamClient, utils::callbacks::SteamShutdownDispatcher};

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
    pub(crate) steam_shutdown_callback: SteamShutdownDispatcher,
}

pub(crate) trait CallbackDispatcher: Send + Sync {
    type Item: CallbackTyped;

    fn storage(&self) -> &Mutex<Option<Sender<Self::Item>>>;

    fn register(&self) -> BroadcastStream<Self::Item> {
        let storage = self.storage();
        let mut guard = storage.lock();

        if let Some(storage) = &*guard {
            BroadcastStream::new(storage.subscribe())
        } else {
            let (sender, receiver) = tokio::sync::broadcast::channel(32);
            *guard = Some(sender);

            BroadcastStream::new(receiver)
        }
    }

    fn proceed(&self, value: Self::Item) {
        let storage = self.storage();
        let mut guard = storage.lock();

        if let Some(storage) = &*storage.lock() {
            match storage.send(value) {
                Ok(_n) => { /* TODO: Log all is okey; number of listeners _n */ }
                Err(_) => {
                    // Clear storage
                    guard.take();
                }
            }
        } else {
            // TODO: Log no listeners
        }
    }
}
