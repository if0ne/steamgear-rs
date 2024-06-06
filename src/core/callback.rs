use std::future::Future;

use futures::{channel::mpsc::UnboundedSender, Stream};
use parking_lot::RwLock;
use steamgear_sys as sys;

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

pub(crate) trait CallbackTyped {
    const TYPE: u32;
    type Raw: Sized;

    fn from_raw(raw: Self::Raw) -> Self;
}

#[derive(Default)]
pub(crate) struct CallbackDispatcher {
    steam_shutdown: RwLock<Vec<UnboundedSender<SteamShutdown>>>,
}

impl CallbackDispatcher {
    pub(crate) fn proceed(&self, callback: sys::CallbackMsg_t) {
        todo!()
    }

    pub(crate) fn register_call_back<T: CallbackTyped>(&self) -> ()/*impl Stream<Item = T> + Send*/ {
       todo!()
    }
}

impl std::fmt::Debug for CallbackDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackDispatcher").finish()
    }
}
