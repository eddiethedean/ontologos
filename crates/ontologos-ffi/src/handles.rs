//! Opaque handle table with generation IDs and serialized access.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const TAG_ONTOLOGY: u32 = 1;
const TAG_BUILDER: u32 = 2;
const TAG_REASONER: u32 = 3;

struct Slot {
    generation: u32,
    ptr: *mut u8,
    drop_fn: Option<unsafe fn(*mut u8)>,
}

impl Drop for Slot {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn.take()
            && !self.ptr.is_null()
        {
            unsafe { drop_fn(self.ptr) };
        }
    }
}

struct TypedSlots {
    next_id: u32,
    slots: HashMap<u32, Slot>,
}

impl TypedSlots {
    fn new() -> Self {
        Self {
            next_id: 1,
            slots: HashMap::new(),
        }
    }

    fn insert<T>(&mut self, tag: u32, value: T) -> i64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let generation = 1;
        let ptr = Box::into_raw(Box::new(value)) as *mut u8;
        self.slots.insert(
            id,
            Slot {
                generation,
                ptr,
                drop_fn: Some(drop_box::<T>),
            },
        );
        encode(tag, generation, id)
    }

    fn with<T, F, R>(&mut self, generation: u32, slot: u32, f: F) -> Result<R, ()>
    where
        F: FnOnce(&mut T) -> R,
    {
        let entry = self.slots.get_mut(&slot).ok_or(())?;
        if entry.generation != generation || entry.ptr.is_null() {
            return Err(());
        }
        let typed = unsafe { &mut *(entry.ptr as *mut T) };
        Ok(f(typed))
    }

    fn drop_slot(&mut self, generation: u32, slot: u32) -> bool {
        let Some(entry) = self.slots.get_mut(&slot) else {
            return false;
        };
        if entry.generation != generation || entry.ptr.is_null() {
            return false;
        }
        if let Some(drop_fn) = entry.drop_fn.take() {
            unsafe { drop_fn(entry.ptr) };
        }
        entry.ptr = std::ptr::null_mut();
        entry.generation = entry.generation.wrapping_add(1);
        true
    }
}

unsafe fn drop_box<T>(ptr: *mut u8) {
    drop(unsafe { Box::from_raw(ptr as *mut T) });
}

struct HandleStore {
    ontologies: TypedSlots,
    builders: TypedSlots,
    reasoners: TypedSlots,
}

impl HandleStore {
    fn new() -> Self {
        Self {
            ontologies: TypedSlots::new(),
            builders: TypedSlots::new(),
            reasoners: TypedSlots::new(),
        }
    }
}

// All access is serialized through `store()`; raw pointers are never used concurrently.
unsafe impl Send for HandleStore {}
unsafe impl Sync for HandleStore {}

fn store() -> &'static Mutex<HandleStore> {
    static STORE: OnceLock<Mutex<HandleStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HandleStore::new()))
}

fn encode(tag: u32, generation: u32, slot: u32) -> i64 {
    if slot == 0 {
        return 0;
    }
    (((tag as u64) << 56) | ((generation as u64) << 32) | (slot as u64)) as i64
}

fn decode(handle: i64) -> Option<(u32, u32, u32)> {
    if handle == 0 {
        return None;
    }
    let raw = handle as u64;
    let slot = (raw & 0xFFFF_FFFF) as u32;
    let generation = ((raw >> 32) & 0x00FF_FFFF) as u32;
    let tag = (raw >> 56) as u32;
    if slot == 0 {
        return None;
    }
    Some((tag, generation, slot))
}

/// Insert an ontology and return an opaque handle.
pub fn into_ontology_handle(value: ontologos_js::JsOntology) -> i64 {
    let mut store = store().lock().expect("handle store poisoned");
    store.ontologies.insert(TAG_ONTOLOGY, value)
}

/// Insert a builder and return an opaque handle.
pub fn into_builder_handle(value: ontologos_js::JsOntologyBuilder) -> i64 {
    let mut store = store().lock().expect("handle store poisoned");
    store.builders.insert(TAG_BUILDER, value)
}

/// Insert a reasoner and return an opaque handle.
pub fn into_reasoner_handle(value: ontologos_js::JsReasoner) -> i64 {
    let mut store = store().lock().expect("handle store poisoned");
    store.reasoners.insert(TAG_REASONER, value)
}

/// Run `f` with a locked ontology reference.
pub fn with_ontology<F, R>(handle: i64, f: F) -> Result<R, ()>
where
    F: FnOnce(&mut ontologos_js::JsOntology) -> R,
{
    let (tag, generation, slot) = decode(handle).ok_or(())?;
    if tag != TAG_ONTOLOGY {
        return Err(());
    }
    let mut store = store().lock().map_err(|_| ())?;
    store.ontologies.with(generation, slot, f)
}

/// Run `f` with a locked builder reference.
pub fn with_builder<F, R>(handle: i64, f: F) -> Result<R, ()>
where
    F: FnOnce(&mut ontologos_js::JsOntologyBuilder) -> R,
{
    let (tag, generation, slot) = decode(handle).ok_or(())?;
    if tag != TAG_BUILDER {
        return Err(());
    }
    let mut store = store().lock().map_err(|_| ())?;
    store.builders.with(generation, slot, f)
}

/// Run `f` with a locked reasoner reference.
pub fn with_reasoner<F, R>(handle: i64, f: F) -> Result<R, ()>
where
    F: FnOnce(&mut ontologos_js::JsReasoner) -> R,
{
    let (tag, generation, slot) = decode(handle).ok_or(())?;
    if tag != TAG_REASONER {
        return Err(());
    }
    let mut store = store().lock().map_err(|_| ())?;
    store.reasoners.with(generation, slot, f)
}

/// Drop an ontology handle; returns false if stale or invalid.
pub fn drop_ontology_handle(handle: i64) -> bool {
    let Some((tag, generation, slot)) = decode(handle) else {
        return false;
    };
    if tag != TAG_ONTOLOGY {
        return false;
    }
    let mut store = store().lock().expect("handle store poisoned");
    store.ontologies.drop_slot(generation, slot)
}

/// Drop a builder handle; returns false if stale or invalid.
pub fn drop_builder_handle(handle: i64) -> bool {
    let Some((tag, generation, slot)) = decode(handle) else {
        return false;
    };
    if tag != TAG_BUILDER {
        return false;
    }
    let mut store = store().lock().expect("handle store poisoned");
    store.builders.drop_slot(generation, slot)
}

/// Drop a reasoner handle; returns false if stale or invalid.
pub fn drop_reasoner_handle(handle: i64) -> bool {
    let Some((tag, generation, slot)) = decode(handle) else {
        return false;
    };
    if tag != TAG_REASONER {
        return false;
    }
    let mut store = store().lock().expect("handle store poisoned");
    store.reasoners.drop_slot(generation, slot)
}
