use async_trait::async_trait;
use crate::core::pipeline::argument::Argument;
use crate::core::pipeline::modifier::Modifier;
use crate::core::pipeline::context::Context;

#[derive(Debug, Clone)]
pub struct LteModifier {
    argument: Argument
}

impl LteModifier {
    pub fn new(argument: impl Into<Argument>) -> Self {
        Self { argument: argument.into() }
    }
}

#[async_trait]
impl Modifier for LteModifier {

    fn name(&self) -> &'static str {
        "lte"
    }

    async fn call(&self, ctx: Context) -> Context {
        let rhs = self.argument.resolve(ctx.clone()).await;
        if ctx.value <= rhs {
            ctx
        } else {
            ctx.invalid("Value is not less than or equal to rhs.")
        }
    }
}