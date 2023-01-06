use std::sync::Arc;
use crate::core::pipeline::modifier::Modifier;
use crate::core::pipeline::modifiers::array::get_length::GetLengthModifier;
use crate::core::pipeline::modifiers::bcrypt::bcrypt_salt::BcryptSaltModifier;
use crate::core::pipeline::modifiers::bcrypt::bcrypt_verify::BcryptVerifyModifier;
use crate::parser::ast::argument::Argument;

pub(crate) fn bcrypt_verify(args: Vec<Argument>) -> Arc<dyn Modifier> {
    let value = args.get(0).unwrap().resolved.as_ref().unwrap().as_value().unwrap();
    Arc::new(BcryptVerifyModifier::new(value.as_pipeline().unwrap().clone()))
}
