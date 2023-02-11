use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::teon::Value;

use crate::core::pipeline::ctx::Ctx;

#[derive(Debug, Clone)]
pub struct TruncateModifier {
    argument: Value,
}

impl TruncateModifier {
    pub fn new(argument: impl Into<Value>) -> Self {
        Self {
            argument: argument.into(),
        }
    }
}

#[async_trait]
impl Item for TruncateModifier {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        let argument = self.argument.resolve(ctx.clone()).await.as_usize().unwrap();
        match &ctx.value {
            Value::String(s) => ctx.with_value(Value::String(s.chars().take(argument).collect())),
            Value::Vec(v) => ctx.with_value(Value::Vec(v.iter().take(argument).map(|v| v.clone()).collect())),
            _ => ctx.invalid("Value is not string or vector.")
        }
    }
}