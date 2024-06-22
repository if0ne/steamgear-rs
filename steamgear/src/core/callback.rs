use async_channel::{Receiver, Sender};
use parking_lot::Mutex;
use thiserror::Error;

use dashmap::DashMap;

use steamgear_sys as sys;

use crate::{
    apps::callbacks::{DlcInstalled, NewUrlLaunchParams},
    utils::callbacks::SteamShutdown,
};

pub(crate) trait CallbackTyped: Clone + Send + 'static {
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

#[derive(Debug, Default)]
pub(crate) struct ClientCallbackContainer {
    pub(crate) call_results: DashMap<sys::SteamAPICall_t, Sender<sys::CallbackMsg_t>>,

    pub(crate) steam_shutdown_callback: MultiDispatcher<SteamShutdown>,

    // Steam Apps Callbacks
    pub(crate) dlc_installed_callback: OneshotDispatcher<DlcInstalled>,
    pub(crate) new_url_launch_params_callback: MultiDispatcher<NewUrlLaunchParams>,
}

unsafe impl Send for ClientCallbackContainer {}
unsafe impl Sync for ClientCallbackContainer {}

impl ClientCallbackContainer {
    pub(crate) async fn register_call_result<T: CallbackTyped>(
        &self,
        id: sys::SteamAPICall_t,
    ) -> T::Mapped {
        let (sender, receiver) = async_channel::bounded(1);
        self.call_results.insert(id, sender);
        let result = receiver.recv().await.expect("Client dropped");

        assert_eq!(std::mem::size_of::<T::Raw>(), result.m_cubParam as usize);

        let raw_data = unsafe { T::from_ptr(result.m_pubParam) };
        T::from_raw(raw_data)
    }
}

pub(crate) trait CallbackDispatcher: Send + Sync {
    type Item: CallbackTyped;
    type Output<'a>
    where
        Self: 'a;

    fn register(&self) -> Self::Output<'_>;
    fn proceed(&self, value: Self::Item);
}

#[derive(Debug)]
pub(crate) struct SingleDispatcher<T: CallbackTyped> {
    inner: Mutex<Option<Sender<T>>>,
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
    type Output<'a> = Receiver<Self::Item>;

    fn register(&self) -> Self::Output<'_> {
        let storage = &self.inner;
        let mut guard = storage.lock();
        let (sender, receiver) = async_channel::bounded(8);

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

        if let Some(sender) = &mut *guard {
            match sender.send_blocking(value) {
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
    type Output<'a> = Receiver<Self::Item>;

    fn register(&self) -> Self::Output<'_> {
        let storage = &self.inner;
        let mut guard = storage.lock();
        let (sender, receiver) = async_channel::bounded(1);

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
            match sender.send_blocking(value) {
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
pub(crate) struct MultiDispatcher<T: CallbackTyped> {
    inner: (Sender<T>, Receiver<T>),
}

impl<T: CallbackTyped> Default for MultiDispatcher<T> {
    fn default() -> Self {
        let (sender, receiver) = async_channel::bounded(32);
        Self {
            inner: (sender, receiver),
        }
    }
}

impl<T: CallbackTyped> CallbackDispatcher for MultiDispatcher<T> {
    type Item = T;
    type Output<'a> = Receiver<Self::Item>;

    fn register(&self) -> Self::Output<'_> {
        let (_, recv) = &self.inner;
        recv.clone()
    }

    fn proceed(&self, value: Self::Item) {
        let (sender, _) = &self.inner;

        if sender.receiver_count() > 1 {
            match sender.send_blocking(value) {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub(crate) enum CallbackType {
    SteamShutdown = sys::SteamShutdown_t_k_iCallback as u32,
    FileDetailsResult = sys::FileDetailsResult_t_k_iCallback as u32,
    DlcInstalled = sys::DlcInstalled_t_k_iCallback as u32,
    NewUrlLaunchParameters = sys::NewUrlLaunchParameters_t_k_iCallback as u32,
}

impl CallbackType {
    // Do not use placeholder
    pub(crate) fn is_for_client(&self) -> bool {
        match self {
            CallbackType::SteamShutdown => true,
            CallbackType::FileDetailsResult => true,
            CallbackType::DlcInstalled => true,
            CallbackType::NewUrlLaunchParameters => true,
        }
    }

    // Do not use placeholder
    pub(crate) fn is_for_server(&self) -> bool {
        match self {
            CallbackType::SteamShutdown => true,
            CallbackType::FileDetailsResult => false,
            CallbackType::DlcInstalled => false,
            CallbackType::NewUrlLaunchParameters => false,
        }
    }
}

#[derive(Clone, Error, Debug)]
pub enum CallbackError {
    #[error("This callback is pending elsewhere, the current request is canceled")]
    Canceled,
}
