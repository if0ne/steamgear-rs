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
        callback: impl FnMut(&T) + Send + 'static,
    ) -> CallbackId<T> {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let any: Box<dyn std::any::Any> = Box::new(callback);

        self.map
            .entry(std::any::TypeId::of::<T>())
            .and_modify(|map| {
                map.insert(id, any);
            })
            .or_insert(HashMap::from_iter([(id, any)]));

        CallbackId(id, PhantomData)
    }
}
