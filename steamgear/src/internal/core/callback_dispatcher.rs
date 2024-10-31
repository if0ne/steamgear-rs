use std::{collections::HashMap, marker::PhantomData, sync::atomic::AtomicU32};

use dashmap::DashMap;

use crate::structs::CallbackId;

use super::call_result::CallResultTyped;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

pub(crate) struct CallbackDispatcher {
    map: DashMap<std::any::TypeId, HashMap<u32, Box<dyn std::any::Any>>>,
}

impl CallbackDispatcher {
    pub(crate) fn register<T: CallResultTyped>(
        &self,
        callback: impl Dispatchable<T>,
    ) -> CallbackId<T> {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let callback = Box::new(callback);
        let any: Box<dyn std::any::Any> = Box::new(callback);

        match self.map.entry(std::any::TypeId::of::<T>()) {
            dashmap::mapref::entry::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().insert(id, any);
            }
            dashmap::mapref::entry::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(HashMap::from([(id, any)]));
            }
        };

        CallbackId(id, PhantomData)
    }

    pub(crate) fn unregister<T: CallResultTyped>(&self, id: CallbackId<T>) {
        self.map.entry(std::any::TypeId::of::<T>()).and_modify(|v| {
            v.remove(&id.0);
        });
    }

    pub(crate) fn invoke<T: CallResultTyped>(&self, data: &T) {
        self.map.entry(std::any::TypeId::of::<T>()).and_modify(|v| {
            for (_, cb) in v {
                if let Some(cb) = cb.downcast_mut::<Box<dyn Dispatchable<T>>>() {
                    cb.invoke(data);
                }
            }
        });
    }
}

pub trait Dispatchable<T: CallResultTyped>: Send + 'static {
    fn invoke(&mut self, data: &T);
}

impl<T: CallResultTyped, F> Dispatchable<T> for F
where
    F: FnMut(&T) + Send + 'static,
{
    fn invoke(&mut self, data: &T) {
        (*self)(data)
    }
}
