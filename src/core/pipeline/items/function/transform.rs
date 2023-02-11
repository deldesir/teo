use std::fmt::{Debug, Formatter};
use std::future::Future;

use std::sync::Arc;
use async_trait::async_trait;
use futures_util::future::BoxFuture;

use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::core::teon::Value;

pub trait TransformArgument<T: From<Value> + Send + Sync>: Send + Sync {
    fn call(&self, args: T) -> BoxFuture<'static, T>;
}

impl<T, F, Fut> TransformArgument<T> for F where
    T: From<Value> + Send + Sync,
    F: Fn(T) -> Fut + Sync + Send,
    Fut: Future<Output = T> + Send + Sync + 'static {
    fn call(&self, args: T) -> BoxFuture<'static, T> {
        Box::pin(self(args))
    }
}

#[derive(Clone)]
pub struct TransformModifier<T> {
    callback: Arc<dyn TransformArgument<T>>
}

impl<T> Debug for TransformModifier<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct("TransformModifier");
        result.finish()
    }
}

impl<T> TransformModifier<T> {
    pub fn new<F>(f: F) -> TransformModifier<T> where
        T: From<Value> + Send + Sync,
        F: TransformArgument<T> + 'static {
        return TransformModifier {
            callback: Arc::new(f)
        }
    }
}

#[async_trait]
impl<T: Into<Value> + From<Value> + Send + Sync> Item for TransformModifier<T> {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        let cb = self.callback.clone();
        let value = cb.call((&ctx).value.clone().into()).await;
        ctx.with_value(value.into())
    }
}

unsafe impl<T> Send for TransformModifier<T> {}
unsafe impl<T> Sync for TransformModifier<T> {}