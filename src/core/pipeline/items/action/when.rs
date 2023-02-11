use async_trait::async_trait;
use crate::core::action::Action;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::Pipeline;
use crate::core::pipeline::ctx::Ctx;

#[derive(Debug, Clone)]
pub struct WhenModifier {
    actions: Vec<Action>,
    pipeline: Pipeline
}

impl WhenModifier {
    pub(crate) fn new(actions: Vec<Action>, pipeline: Pipeline) -> Self {
        return WhenModifier {
            actions,
            pipeline
        };
    }
}

#[async_trait]
impl Item for WhenModifier {

    async fn call<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        let object_action = ctx.object.as_ref().unwrap().action();
        if object_action.passes(&self.actions) {
            self.pipeline.process(ctx.clone()).await
        } else {
            ctx
        }
    }
}