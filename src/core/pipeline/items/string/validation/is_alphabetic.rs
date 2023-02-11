use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;

#[derive(Debug, Copy, Clone)]
pub struct IsAlphabeticModifier {}

impl IsAlphabeticModifier {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Item for IsAlphabeticModifier {
    async fn call<'a>(&self, context: Ctx<'a>) -> Ctx<'a> {
        match context.value.as_str() {
            None => context.invalid("Value is not string."),
            Some(s) => {
                for c in s.chars() {
                    if !c.is_alphabetic() {
                        return context.invalid("Value is not alphabetic.");
                    }
                }
                context
            }
        }
    }
}