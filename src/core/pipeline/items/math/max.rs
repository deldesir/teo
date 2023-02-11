use std::cmp::{min};
use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::prelude::Value;

#[derive(Debug, Clone)]
pub struct MaxModifier {
    argument: Value
}

impl MaxModifier {
    pub fn new(argument: impl Into<Value>) -> Self {
        Self { argument: argument.into() }
    }
}

#[async_trait]
impl Item for MaxModifier {

    async fn call<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        let argument = self.argument.resolve(ctx.clone()).await;
        match ctx.value {
            Value::I32(v) => ctx.with_value(Value::I32(min(v, argument.as_i32().unwrap()))),
            Value::I64(v) => ctx.with_value(Value::I64(min(v, argument.as_i64().unwrap()))),
            Value::F32(v) => ctx.with_value(Value::F32(if v <= argument.as_f32().unwrap() { v } else { argument.as_f32().unwrap() })),
            Value::F64(v) => ctx.with_value(Value::F64(if v <= argument.as_f64().unwrap() { v } else { argument.as_f64().unwrap() })),
            _ => ctx.invalid("Value is not number."),
        }
    }
}