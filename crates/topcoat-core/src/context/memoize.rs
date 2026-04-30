use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

use bumpalo::Bump;
use tokio::sync::OnceCell;

use crate::context::Cx;

struct MemoizeCache<K, V> {}

impl<K, V> MemoizeCache<K, V> {
    fn bump() -> MutexGuard<'static, Bump> {
        static BUMP: LazyLock<Mutex<Bump>> = LazyLock::new(|| Mutex::new(Bump::new()));
        BUMP.lock().unwrap()
    }

    fn kek() -> &'static mut str {
        let kek = Self::bump();
        let lel = kek.alloc_str("smep");
        lel
    }
}

pub fn memoize_raw<'a, K, V, F>(cx: &'a Cx, key: K, f: F) -> Memoized<'a, V>
where
    K: DynKey + Clone,
    V: Send + Sync + 'static,
    F: FnOnce(K) -> V,
{
    Memoized {
        inner: cx.cache.memoize(key, f),
        lifetime: Default::default(),
    }
}

pub async fn memoize_raw_async<'a, K, V, F, Fut>(cx: &'a Cx, key: K, f: F) -> Memoized<'a, V>
where
    K: DynKey + Clone,
    V: Send + Sync + 'static,
    F: FnOnce(K) -> Fut,
    Fut: Future<Output = V>,
{
    Memoized {
        inner: cx.cache.memoize_async(key, f).await,
        lifetime: Default::default(),
    }
}

pub struct Memoized<'a, T> {
    inner: Arc<T>,
    // We artificially limit the lifetime of a memoized value to be the lifetime of the request
    // context. This is because the `Arc` is an implementation detail of the cache. The user should
    // not be able to hold on to the memoized value as long as they want. Conceptually, the cache
    // only lasts as long as the request context. The implementation might change to be more
    // efficient in the future.
    lifetime: PhantomData<&'a ()>,
}

impl<'a, T> Deref for Memoized<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(super) struct DynCache {
    entries: Mutex<HashMap<Box<dyn DynKey>, Arc<dyn Any + Send + Sync>>>,
}

impl DynCache {
    pub(super) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn memoize<K, V, F>(&self, key: K, f: F) -> Arc<V>
    where
        K: DynKey + Clone,
        V: Send + Sync + 'static,
        F: FnOnce(K) -> V,
    {
        let mut guard = self.entries.lock().unwrap();
        if let Some(value) = guard.get(&key as &dyn DynKey) {
            value
                .clone()
                .downcast::<V>()
                .expect("wrong value type used for cache lookup")
        } else {
            let value = Arc::new(f(key.clone()));
            guard.insert(Box::new(key), value.clone());
            value
        }
    }

    async fn memoize_async<K, V, F, Fut>(&self, key: K, f: F) -> Arc<V>
    where
        K: DynKey + Clone,
        V: Send + Sync + 'static,
        F: FnOnce(K) -> Fut,
        Fut: Future<Output = V>,
    {
        let cell: Arc<OnceCell<Arc<V>>> = {
            let mut guard = self.entries.lock().unwrap();
            if let Some(value) = guard.get(&key as &dyn DynKey) {
                value
                    .clone()
                    .downcast::<OnceCell<Arc<V>>>()
                    .expect("wrong value type used for cache lookup")
            } else {
                let cell = Arc::new(OnceCell::<Arc<V>>::new());
                guard.insert(Box::new(key.clone()), cell.clone());
                cell
            }
        };
        cell.get_or_init(|| async { Arc::new(f(key).await) })
            .await
            .clone()
    }
}

impl std::fmt::Debug for DynCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynRequestCache").finish()
    }
}

pub trait DynKey: Any + Send + Sync {
    fn dyn_eq(&self, other: &dyn DynKey) -> bool;
    fn dyn_hash(&self, state: &mut dyn Hasher);
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Eq + Hash + Send + Sync> DynKey for T {
    fn dyn_eq(&self, other: &dyn DynKey) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<T>().hash(&mut state);
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for dyn DynKey {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for dyn DynKey {}

impl Hash for dyn DynKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}
