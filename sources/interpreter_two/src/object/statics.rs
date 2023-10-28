use std::collections::HashMap;

use super::RuntimeValue;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct StaticFieldRef {
    class: String,
    field_name: String,
    field_descriptor: String,
}

impl StaticFieldRef {
    pub fn new_str(
        class: &'static str,
        field_name: &'static str,
        field_descriptor: &'static str,
    ) -> Self {
        Self {
            class: class.to_string(),
            field_descriptor: field_descriptor.to_string(),
            field_name: field_name.to_string(),
        }
    }

    pub fn new(class: String, field_name: String, field_descriptor: String) -> Self {
        Self {
            class,
            field_descriptor,
            field_name,
        }
    }
}

pub struct StaticFields {
    fields: HashMap<StaticFieldRef, RuntimeValue>,
}

impl StaticFields {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn get_field(&self, field: StaticFieldRef) -> Option<RuntimeValue> {
        self.fields.get(&field).cloned()
    }

    pub fn set_field(
        &mut self,
        field: StaticFieldRef,
        value: RuntimeValue,
    ) -> Option<RuntimeValue> {
        self.fields.insert(field, value)
    }
}
