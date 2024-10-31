use std::{cell::Cell, future::Future, marker::PhantomData};

use steamgear_sys as sys;

use super::{reactor::REACTOR, utils::CallUtils};

#[derive(Debug)]
pub(crate) struct CallResult<T: CallResultTyped> {
    pub(crate) call: sys::SteamAPICall_t,
    pub(crate) utils: CallUtils,
    pub(crate) registered: Cell<bool>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: CallResultTyped> CallResult<T> {
    pub(crate) fn new(call: sys::SteamAPICall_t) -> Self {
        Self {
            call,
            utils: CallUtils::new(),
            registered: Cell::new(false),
            _marker: PhantomData,
        }
    }
}

impl<T: CallResultTyped> Future for CallResult<T> {
    type Output = T::Mapped;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.registered.get() {
            REACTOR.add(self.call, cx.waker().clone());
            self.registered.set(true);
        }

        if let Some(true) = self.utils.is_api_call_completed(self.call) {
            let result = self
                .utils
                .get_api_call_result::<T>(self.call)
                .expect("Call result must be completed");

            REACTOR.remove(self.call);

            std::task::Poll::Ready(result)
        } else {
            std::task::Poll::Pending
        }
    }
}

pub(crate) trait CallResultTyped: Clone + Send + 'static {
    const TYPE: CallbackType;
    type Raw: Copy;
    type Mapped;

    fn from_raw(raw: Self::Raw) -> Self::Mapped;

    unsafe fn from_ptr(ptr: *mut u8) -> Self::Raw {
        assert_eq!(
            std::mem::align_of::<Self::Raw>(),
            std::mem::align_of_val(&ptr)
        );

        let raw_type: Self::Raw = *(ptr as *const Self::Raw);

        raw_type
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub(crate) enum CallbackType {
    SteamShutdown = sys::SteamShutdown_t_k_iCallback as u32,
    FileDetailsResult = sys::FileDetailsResult_t_k_iCallback as u32,
    DlcInstalled = sys::DlcInstalled_t_k_iCallback as u32,
    NewUrlLaunchParameters = sys::NewUrlLaunchParameters_t_k_iCallback as u32,
}
