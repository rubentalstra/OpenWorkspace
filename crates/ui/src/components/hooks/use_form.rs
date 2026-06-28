use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use validator::Validate;

/// Marker for types usable as form models: validatable, round-trippable through
/// JSON, and `'static` so they can back reactive signals.
///
/// A blanket impl covers every type meeting the bounds; it is not implemented
/// by hand.
pub trait FormData:
    Validate + Clone + Default + Serialize + for<'de> Deserialize<'de> + 'static
{
}

impl<T> FormData for T where
    T: Validate + Clone + Default + Serialize + for<'de> Deserialize<'de> + 'static
{
}

/// Boxed setter handed to field components to write a single field's raw value.
pub type SetValueFn = Box<dyn Fn(&str, String) + Send + Sync>;

/// Boxed callback handed to field components to mark a field touched on blur.
pub type TouchFieldFn = Box<dyn Fn(&str) + Send + Sync>;

/// Reactive form state for the model `T`: raw string values keyed by field
/// name, per-field validation errors, and the set of touched (blurred) fields.
///
/// `Copy`, so it can be captured by value into closures and child components.
pub struct Form<T> {
    /// Raw, unparsed string value entered for each field, keyed by field name.
    pub values_signal: RwSignal<HashMap<String, String>>,
    /// Current validation message per field; `None` means the field is valid.
    pub errors_signal: RwSignal<HashMap<String, Option<String>>>,
    /// Names of fields the user has blurred at least once.
    pub touched_signal: RwSignal<HashSet<String>>,
    phantom: PhantomData<T>,
}

impl<T> Clone for Form<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Form<T> {}

impl<T> Default for Form<T> {
    fn default() -> Self {
        Self {
            values_signal: RwSignal::new(HashMap::new()),
            errors_signal: RwSignal::new(HashMap::new()),
            touched_signal: RwSignal::new(HashSet::new()),
            phantom: PhantomData,
        }
    }
}

impl<T> Form<T>
where
    T: FormData,
{
    /// Returns the current raw string value of `field`, or empty if unset.
    pub fn value(&self, field: &str) -> String {
        self.values_signal
            .read()
            .get(field)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns the current validation error for `field`, if any.
    pub fn error(&self, field: &str) -> Option<String> {
        self.errors_signal.read().get(field).and_then(Clone::clone)
    }

    /// Writes `field`'s raw value. Re-validates only fields already touched, so
    /// errors do not appear before the user leaves a field for the first time.
    pub fn set_value(&self, field: &str, value: String) {
        let field = field.to_string();

        self.values_signal.update(|values| {
            values.insert(field.clone(), value);
        });

        if self.is_touched(&field) {
            let error = self.validate_field(&field);
            self.errors_signal.update(|errors| {
                errors.insert(field, error);
            });
        }
    }

    /// Marks `field` touched (on blur) and validates it. Errors are surfaced
    /// only for touched fields.
    pub fn touch_field(&self, field: &str) {
        let field = field.to_string();

        self.touched_signal.update(|touched| {
            touched.insert(field.clone());
        });

        let error = self.validate_field(&field);
        self.errors_signal.update(|errors| {
            errors.insert(field, error);
        });
    }

    /// Returns whether `field` has been blurred at least once.
    pub fn is_touched(&self, field: &str) -> bool {
        self.touched_signal.read().contains(field)
    }

    fn validate_field(&self, field: &str) -> Option<String> {
        let data = Self::map_to_struct(&self.values_signal.read())?;
        let errors = data.validate().err()?;
        let message = errors
            .field_errors()
            .get(field)?
            .first()?
            .message
            .as_ref()?;
        Some(message.to_string())
    }

    /// Builds `T` by overlaying the raw string values onto `T`'s default,
    /// coercing each value to a JSON number when it parses as one. Empty values
    /// keep the struct default so partially filled forms still deserialize.
    fn map_to_struct(values: &HashMap<String, String>) -> Option<T> {
        let default_value = serde_json::to_value(T::default()).ok()?;
        let mut field_map: HashMap<String, Value> = serde_json::from_value(default_value).ok()?;

        for (key, value) in values {
            if value.is_empty() {
                continue;
            }

            let json_value = if let Ok(int) = value.parse::<i64>() {
                Value::Number(int.into())
            } else if let Ok(float) = value.parse::<f64>() {
                Number::from_f64(float).map_or_else(|| Value::String(value.clone()), Value::Number)
            } else {
                Value::String(value.clone())
            };
            field_map.insert(key.clone(), json_value);
        }

        serde_json::from_value(serde_json::to_value(field_map).ok()?).ok()
    }

    /// Returns whether every recorded field error is currently clear. Reflects
    /// only fields that have been validated, not unvisited ones.
    pub fn is_valid(&self) -> bool {
        self.errors_signal.read().values().all(Option::is_none)
    }

    /// Returns whether the full model validates, regardless of touched state.
    pub fn can_submit(&self) -> bool {
        let Some(data) = Self::map_to_struct(&self.values_signal.read()) else {
            return false;
        };
        data.validate().is_ok()
    }

    /// Clears all values, errors, and touched state.
    pub fn reset(&self) {
        self.values_signal.set(HashMap::new());
        self.errors_signal.set(HashMap::new());
        self.touched_signal.set(HashSet::new());
    }

    /// Returns the model assembled from current values, or `None` if assembly
    /// fails. Does not run validation.
    pub fn get_data(&self) -> Option<T> {
        Self::map_to_struct(&self.values_signal.read())
    }

    /// Validates and returns the model, or a comma-joined message of the first
    /// error per failing field.
    pub fn validate_and_get(&self) -> Result<T, String> {
        let data = Self::map_to_struct(&self.values_signal.read())
            .ok_or_else(|| "Please fill in all required fields.".to_string())?;

        data.validate().map_err(|errors| {
            errors
                .field_errors()
                .values()
                .filter_map(|errs| errs.first())
                .filter_map(|err| err.message.as_ref())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        })?;

        Ok(data)
    }
}

/// Creates an empty [`Form`] for the model `T`.
pub fn use_form<T>() -> Form<T>
where
    T: FormData,
{
    Form::default()
}

/// Type-erased form state shared through context so generic field components
/// can read and mutate the parent [`Form`] without naming its model type.
#[derive(Clone, Copy)]
pub struct FormContext {
    /// Raw, unparsed string value entered for each field, keyed by field name.
    pub values_signal: RwSignal<HashMap<String, String>>,
    /// Current validation message per field; `None` means the field is valid.
    pub errors_signal: RwSignal<HashMap<String, Option<String>>>,
    /// Names of fields the user has blurred at least once.
    pub touched_signal: RwSignal<HashSet<String>>,
    /// Writes a single field's raw value into the parent form.
    pub set_value: StoredValue<SetValueFn>,
    /// Marks a single field touched in the parent form.
    pub touch_field: StoredValue<TouchFieldFn>,
}

/// Identifies which model field a field component is bound to, provided via
/// context to its descendants.
#[derive(Clone)]
pub struct FieldContext {
    /// Model field name this component reads from and writes to.
    pub name: String,
}

/// Lets a model render its own form fields, wired to a [`Form`]. Implement it by
/// hand on each form model (compose [`FormField`](crate::FormField) with the
/// `Form*` field controls).
pub trait AutoFormFields: FormData {
    /// Renders every form field for the model, wired to `form`.
    fn render_fields(form: Form<Self>) -> impl IntoView;
}
