use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::prelude::{DateTime};
use serde_json::{Value as JsonValue};
use crate::core::argument::Argument;
use crate::core::field::Type;
use crate::core::model::Model;
use crate::core::stage::Stage;
use crate::core::value::Value;
use crate::error::ActionError;


pub struct Object {
    pub model: Arc<Model>,
    pub is_initialized: AtomicBool,
    pub is_new: AtomicBool,
    pub is_modified: AtomicBool,
    pub is_partial: bool,
    pub is_deleted: bool,
    pub selected_fields: RefCell<HashSet<String>>,
    pub modified_fields: RefCell<HashSet<String>>,
    pub previous_values: RefCell<HashMap<String, Value>>,
    pub value_map: RefCell<HashMap<String, Value>>,
}

impl Object {
    pub(crate) fn new(model: Arc<Model>) -> Arc<Object> {
        return Arc::new(Object {
            model,
            is_initialized: AtomicBool::new(false),
            is_new: AtomicBool::new(true),
            is_modified: AtomicBool::new(false),
            is_partial: false,
            is_deleted: false,
            selected_fields: RefCell::new(HashSet::new()),
            modified_fields: RefCell::new(HashSet::new()),
            previous_values: RefCell::new(HashMap::new()),
            value_map: RefCell::new(HashMap::new())
        });
    }

    pub async fn set_json(self: Arc<Object>, json_value: JsonValue) -> Result<Arc<Object>, ActionError> {
        self.set_or_update_json(json_value, true).await
    }

    pub async fn update_json(self: Arc<Object>, json_value: JsonValue) -> Result<Arc<Object>, ActionError> {
        self.set_or_update_json(json_value, false).await
    }

    pub fn set_value(self: Arc<Object>, key: &'static str, value: Value) -> Result<Arc<Object>, ActionError> {
        let model_keys = &self.clone().model.save_keys;
        if !model_keys.contains(&key) {
            return Err(ActionError::keys_unallowed());
        }
        self.value_map.borrow_mut().insert(key.to_string(), value);
        if !self.is_new.load(Ordering::SeqCst) {
            self.is_modified.store(true, Ordering::SeqCst);
            self.modified_fields.borrow_mut().insert(key.to_string());
        }
        return Ok(self)
    }

    pub fn get_value(self: Arc<Object>, key: &'static str) -> Result<Option<Value>, ActionError> {
        let model_keys = &self.clone().model.all_getable_keys; // TODO: should be all keys
        if !model_keys.contains(&key) {
            return Err(ActionError::keys_unallowed());
        }
        match self.value_map.borrow().get(key) {
            Some(value) => {
                Ok(Some(value.clone()))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn select(self: Arc<Object>, keys: HashSet<String>) -> Result<Arc<Object>, ActionError> {
        self.selected_fields.borrow_mut().extend(keys);
        return Ok(self);
    }

    pub fn deselect(self: Arc<Object>, keys: HashSet<String>) -> Result<Arc<Object>, ActionError> {
        if self.selected_fields.borrow().len() == 0 {
            self.selected_fields.borrow_mut().extend(self.model.output_keys.iter().map(|s| { s.to_string()}));
        }
        for key in keys {
            self.selected_fields.borrow_mut().remove(&key);
        }
        return Ok(self);
    }

    pub async fn save(self: Arc<Object>) -> Result<Arc<Object>, ActionError> {
        // apply on save pipeline first
        let model_keys = &self.clone().model.save_keys;
        for key in model_keys {
            let field = self.model.field(&key);
            if field.on_save_pipeline._has_any_modifier() {
                let mut stage = match self.value_map.borrow().deref().get(&key.to_string()) {
                    Some(value) => {
                        Stage::Value(value.clone())
                    }
                    None => {
                        Stage::Value(Value::Null)
                    }
                };
                stage = field.on_save_pipeline._process(stage.clone(), self.clone()).await;
                match stage {
                    Stage::Invalid(s) => {
                        return Err(ActionError::invalid_input(key, s));
                    }
                    Stage::Value(v) => {
                        self.value_map.borrow_mut().insert(key.to_string(), v);
                        if !self.is_new.load(Ordering::SeqCst) {
                            self.is_modified.store(true, Ordering::SeqCst);
                            self.modified_fields.borrow_mut().insert(key.to_string());
                        }
                    }
                    Stage::ConditionTrue(_) => {
                        return Err(ActionError::internal_server_error("Pipeline modifiers are invalid.".to_string()))
                    }
                    Stage::ConditionFalse(_) => {
                        return Err(ActionError::internal_server_error("Pipeline modifiers are invalid.".to_string()))
                    }
                }
            }
        }
        // then do nothing haha
        return Ok(self);
    }

    pub fn delete(&self) -> &Object {
        return self;
    }

    pub fn to_json(&self) -> &Self {
        return self;
    }

    pub async fn include(&self) -> &Object {
        return self;
    }

    async fn set_or_update_json(
        self: Arc<Object>,
        json_value: JsonValue,
        validate_and_transform: bool) -> Result<Arc<Object>, ActionError> {
        let json_object = json_value.as_object().unwrap();
        // check keys first
        let json_keys: Vec<&str> = json_object.keys().map(|k| { k.as_str() }).collect();
        let model_keys = if validate_and_transform {
            &self.model.input_keys
        } else {
            &self.model.save_keys
        };
        let keys_valid = json_keys.iter().all(|&item| model_keys.contains(&item));
        if !keys_valid {
            return Err(ActionError::keys_unallowed());
        }
        // assign values
        let initialized = self.is_initialized.load(Ordering::SeqCst);
        let keys_to_iterate = if initialized { &json_keys } else { model_keys };
        let this = self.clone();
        for key in keys_to_iterate {
            let field = this.model.field(&key);
            let json_has_value = if initialized { true } else {
                json_keys.contains(key)
            };
            if json_has_value {
                let json_value = &json_object[&key.to_string()];
                let mut value = match field.r#type {
                    Type::ObjectId => { Value::ObjectId(json_value.to_string()) }
                    Type::Bool => { Value::Bool(json_value.as_bool().unwrap()) }
                    Type::I8 => { Value::I8(json_value.as_i64().unwrap() as i8) }
                    Type::I16 => { Value::I16(json_value.as_i64().unwrap() as i16) }
                    Type::I32 => { Value::I32(json_value.as_i64().unwrap() as i32) }
                    Type::I64 => { Value::I64(json_value.as_i64().unwrap()) }
                    Type::I128 => { Value::I128(json_value.as_i64().unwrap() as i128) }
                    Type::U8 => { Value::U8(json_value.as_i64().unwrap() as u8) }
                    Type::U16 => { Value::U16(json_value.as_i64().unwrap() as u16) }
                    Type::U32 => { Value::U32(json_value.as_i64().unwrap() as u32) }
                    Type::U64 => { Value::U64(json_value.as_i64().unwrap() as u64) }
                    Type::U128 => { Value::U128(json_value.as_i64().unwrap() as u128) }
                    Type::F32 => { Value::F32(json_value.as_f64().unwrap() as f32) }
                    Type::F64 => { Value::F64(json_value.as_f64().unwrap() as f64) }
                    Type::String => { Value::String(String::from(json_value.as_str().unwrap())) }
                    Type::DateTime => { Value::DateTime(DateTime::from(DateTime::parse_from_rfc3339(&json_value.to_string()).ok().unwrap())) }
                    _ => { panic!() }
                };
                if validate_and_transform {
                    // pipeline
                    let mut stage = Stage::Value(value);
                    stage = field.on_set_pipeline._process(stage.clone(), self.clone()).await;
                    match stage {
                        Stage::Invalid(_) => {
                            return Err(ActionError::keys_unallowed())
                        }
                        Stage::Value(v) => {
                            value = v
                        }
                        Stage::ConditionTrue(_) => {
                            return Err(ActionError::internal_server_error("Pipeline modifiers are invalid.".to_string()))
                        }
                        Stage::ConditionFalse(_) => {
                            return Err(ActionError::internal_server_error("Pipeline modifiers are invalid.".to_string()))
                        }
                    }
                }
                self.value_map.borrow_mut().insert(key.to_string(), value);
                if !self.is_new.load(Ordering::SeqCst) {
                    self.is_modified.store(true, Ordering::SeqCst);
                    self.modified_fields.borrow_mut().insert(key.to_string());
                }
            } else {
                // apply default values
                if !initialized {
                    if let Some(argument) = &field.default {
                        match argument {
                            Argument::ValueArgument(value) => {
                                self.value_map.borrow_mut().insert(key.to_string(), value.clone());
                            }
                            Argument::PipelineArgument(pipeline) => {
                                let stage = pipeline._process(Stage::Value(Value::Null), self.clone()).await;
                                self.value_map.borrow_mut().insert(key.to_string(), stage.value().unwrap());
                            }
                            Argument::FunctionArgument(farg) => {
                                let stage = farg.call(Value::Null, self.clone()).await;
                                self.value_map.borrow_mut().insert(key.to_string(), stage.value().unwrap());
                            }
                        }
                    }
                }
            }
        };
        // set flag
        self.is_initialized.store(true, Ordering::SeqCst);
        Ok(self)
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct(self.model.name);
        for (key, value) in self.value_map.borrow().iter() {
            result.field(key, value);
        }
        result.finish()
    }
}

unsafe impl Send for Object {}
unsafe impl Sync for Object {}
