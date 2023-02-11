use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::core::teon::Value;

#[derive(Debug, Clone)]
pub struct GteModifier {
    argument: Value
}

impl GteModifier {
    pub fn new(argument: impl Into<Value>) -> Self {
        Self { argument: argument.into() }
    }
}

#[async_trait]
impl Item for GteModifier {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        let rhs = self.argument.resolve(ctx.clone()).await;
        if ctx.value >= rhs {
            ctx
        } else {
            ctx.invalid("Value is not greater than or equal to rhs.")
        }
    }
}