use futures::channel::{mpsc, oneshot};
use parking_lot::{Mutex, MutexGuard};
use thiserror::Error;

use dashmap::DashMap;

use steamgear_sys as sys;

use crate::{
    apps::callbacks::{DlcInstalled, NewUrlLaunchParams},
    utils::callbacks::SteamShutdown,
};

pub(crate) trait CallbackTyped: Clone + Send + 'static {
    const TYPE: u32;
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
pub(crate) struct CallbackContainer {
    pub(crate) call_results: DashMap<sys::SteamAPICall_t, oneshot::Sender<sys::CallbackMsg_t>>,
    pub(crate) steam_shutdown_callback: SingleDispatcher<SteamShutdown>,
    pub(crate) dlc_installed_callback: ReusableDispatcher<DlcInstalled>,
    pub(crate) new_url_launch_params_callback: SingleDispatcher<NewUrlLaunchParams>,
}

unsafe impl Send for CallbackContainer {}
unsafe impl Sync for CallbackContainer {}

impl CallbackContainer {
    pub(crate) async fn register_call_result<T: CallbackTyped>(
        &self,
        id: sys::SteamAPICall_t,
    ) -> T::Mapped {
        let (sender, receiver) = futures::channel::oneshot::channel();
        self.call_results.insert(id, sender);
        let result = receiver.await.expect("Client dropped");

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
    inner: Mutex<Option<mpsc::Sender<T>>>,
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
    type Output<'a> = mpsc::Receiver<Self::Item>;

    fn register(&self) -> Self::Output<'_> {
        let storage = &self.inner;
        let mut guard = storage.lock();
        let (sender, receiver) = futures::channel::mpsc::channel(8);

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
            match sender.start_send(value) {
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
    type Output<'a> = oneshot::Receiver<Self::Item>;

    fn register(&self) -> Self::Output<'_> {
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

#[derive(Debug)]
pub(crate) struct ReusableDispatcher<T: CallbackTyped> {
    inner: (Mutex<mpsc::Sender<T>>, Mutex<mpsc::Receiver<T>>),
}

impl<T: CallbackTyped> Default for ReusableDispatcher<T> {
    fn default() -> Self {
        let (sender, receiver) = futures::channel::mpsc::channel(8);
        Self {
            inner: (Mutex::new(sender), Mutex::new(receiver)),
        }
    }
}

impl<T: CallbackTyped> CallbackDispatcher for ReusableDispatcher<T> {
    type Item = T;
    type Output<'a> = MutexGuard<'a, mpsc::Receiver<Self::Item>>;

    fn register(&self) -> Self::Output<'_> {
        let (_, recv) = &self.inner;
        recv.lock()
    }

    fn proceed(&self, value: Self::Item) {
        let (sender, receiver) = &self.inner;

        if receiver.is_locked() {
            match sender.lock().start_send(value) {
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
