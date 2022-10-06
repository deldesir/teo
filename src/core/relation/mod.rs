use crate::core::field::optionality::Optionality;
use crate::core::object::Object;
use crate::core::relation::delete_rule::DeleteRule;
use crate::prelude::Value;

pub mod builder;
pub mod delete_rule;
pub mod disconnect_rule;

#[derive(Debug, Clone)]
pub(crate) struct Relation {
    pub(self) name: String,
    pub(self) localized_name: String,
    pub(self) description: String,
    pub(self) optionality: Optionality,
    pub(self) model: String,
    pub(self) through: Option<String>,
    pub(self) is_vec: bool,
    pub(self) fields: Vec<String>,
    pub(self) references: Vec<String>,
    pub(self) delete_rule: DeleteRule,
    pub(self) has_foreign_key: bool,
}

impl Relation {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn localized_name(&self) -> &str {
        &self.localized_name
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }

    pub(crate) fn optionality(&self) -> &Optionality { &self.optionality }

    pub(crate) fn is_optional(&self) -> bool {
        self.optionality.is_optional()
    }

    pub(crate) fn is_required(&self) -> bool {
        self.optionality.is_required()
    }

    pub(crate) fn model(&self) -> &str {
        &self.model
    }

    pub(crate) fn through(&self) -> Option<&str> {
        self.through.as_deref()
    }

    pub(crate) fn is_vec(&self) -> bool {
        self.is_vec
    }

    pub(crate) fn fields(&self) -> &Vec<String> {
        &self.fields
    }

    pub(crate) fn references(&self) -> &Vec<String> {
        &self.references
    }

    pub(crate) fn local(&self) -> &str {
        self.fields.get(0).unwrap()
    }

    pub(crate) fn foreign(&self) -> &str {
        self.references.get(0).unwrap()
    }

    pub(crate) fn delete_rule(&self) -> DeleteRule {
        self.delete_rule
    }

    pub(crate) fn has_foreign_key(&self) -> bool {
        self.has_foreign_key
    }
}

#[derive(Debug)]
pub(crate) enum RelationManipulation {
    Create(Value),
    ConnectOrCreate(Value),
    Connect(Value),
    Set(Value),
    Update(Value),
    Upsert(Value),
    Disconnect(Value),
    Delete(Value),
}

#[derive(Debug)]
pub(crate) enum RelationConnection {
    Link(Object),
    Unlink(Object),
    UnlinkAndDelete(Object),
}
