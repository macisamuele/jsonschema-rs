#[cfg(feature = "perfect_precision")]
use crate::perfect_precision_number::PerfectPrecisionNumber;
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

// Based on implementation proposed by Sven Marnach:
// https://stackoverflow.com/questions/60882381/what-is-the-fastest-correct-way-to-detect-that-there-are-no-duplicates-in-a-json
#[derive(Clone, Debug)]
pub struct HashedValue<'a>(&'a Value);

impl PartialEq<Self> for HashedValue<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HashedValue(Value::Array(self_)), HashedValue(Value::Array(other_))) => {
                self_ == other_
            }
            (HashedValue(Value::Bool(self_)), HashedValue(Value::Bool(other_))) => self_ == other_,
            (HashedValue(Value::Null), HashedValue(Value::Null)) => true,
            (HashedValue(Value::Number(self_)), HashedValue(Value::Number(other_))) => {
                #[cfg(feature = "perfect_precision")]
                {
                    // This is needed because when perfect_precision feature is used then
                    // serde-json stores the floating point as a literal and so 1.0 and 1.00
                    // will not be equal
                    self_ == other_ || PerfectPrecisionNumber::from(self_) == PerfectPrecisionNumber::from(other_)
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    self_ == other_
                }
            }
            (HashedValue(Value::Object(self_)), HashedValue(Value::Object(other_))) => {
                self_ == other_
            }
            (HashedValue(Value::String(self_)), HashedValue(Value::String(other_))) => {
                self_ == other_
            }

            _ => false,
        }
    }
}
impl Eq for HashedValue<'_> {}

impl Hash for HashedValue<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0 {
            Value::Null => state.write_u32(3_221_225_473), // chosen randomly
            Value::Bool(ref item) => item.hash(state),
            Value::Number(ref item) => {
                #[cfg(feature = "perfect_precision")]
                {
                    PerfectPrecisionNumber::from(item).hash(state)
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    if let Some(number) = item.as_u64() {
                        number.hash(state);
                    } else if let Some(number) = item.as_i64() {
                        number.hash(state);
                    } else if let Some(number) = item.as_f64() {
                        number.to_bits().hash(state)
                    }
                }
            }
            Value::String(ref item) => item.hash(state),
            Value::Array(ref items) => {
                for item in items {
                    HashedValue(item).hash(state);
                }
            }
            Value::Object(ref items) => {
                let mut hash = 0;
                for (key, value) in items {
                    // We have no way of building a new hasher of type `H`, so we
                    // hardcode using the default hasher of a hash map.
                    let mut item_hasher = DefaultHasher::default();
                    key.hash(&mut item_hasher);
                    HashedValue(value).hash(&mut item_hasher);
                    hash ^= item_hasher.finish();
                }
                state.write_u64(hash);
            }
        }
    }
}

#[inline]
pub fn is_unique(items: &[Value]) -> bool {
    let mut seen = HashSet::with_capacity(items.len());
    items.iter().map(HashedValue).all(|x| seen.insert(x))
}

pub struct UniqueItemsValidator {}

impl UniqueItemsValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(UniqueItemsValidator {}))
    }
}

impl Validate for UniqueItemsValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::unique_items(instance)
    }

    fn name(&self) -> String {
        "uniqueItems: true".to_string()
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        is_unique(instance_value)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            self.is_valid_array(schema, instance, instance_value)
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Bool(value) = schema {
        if *value {
            Some(UniqueItemsValidator::compile())
        } else {
            None
        }
    } else {
        None
    }
}
