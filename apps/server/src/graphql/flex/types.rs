//! GraphQL types for the Flex custom fields system.

use async_graphql::{InputObject, SimpleObject};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::models::user_field_definitions::Model;
use flex::{
    FieldDefinitionView, FlexEntryView, FlexSchemaView, SchemaRetroValidationReport,
    EntryDriftDetail,
};

/// GraphQL representation of a field definition.
#[derive(Debug, Clone, SimpleObject)]
pub struct FieldDefinitionObject {
    pub id: Uuid,
    /// Tenant-scoped unique key (snake_case, `^[a-z][a-z0-9_]{0,127}$`).
    pub field_key: String,
    /// Serialised field type value, e.g. `"text"`, `"select"`.
    pub field_type: String,
    /// Localised labels as JSON object: `{"en": "Phone", "ru": "Телефон"}`.
    pub label: JsonValue,
    /// Optional localised description.
    pub description: Option<JsonValue>,
    /// Whether field values belong to locale-aware parallel records.
    pub is_localized: bool,
    pub is_required: bool,
    /// Default value applied by `apply_defaults()`.
    pub default_value: Option<JsonValue>,
    /// Validation constraints as JSON (min, max, pattern, options, …).
    pub validation: Option<JsonValue>,
    /// Display order (ascending).
    pub position: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Model> for FieldDefinitionObject {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            field_key: m.field_key,
            field_type: m.field_type,
            label: m.label,
            description: m.description,
            is_localized: m.is_localized,
            is_required: m.is_required,
            default_value: m.default_value,
            validation: m.validation,
            position: m.position,
            is_active: m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

impl From<FieldDefinitionView> for FieldDefinitionObject {
    fn from(m: FieldDefinitionView) -> Self {
        Self {
            id: m.id,
            field_key: m.field_key,
            field_type: m.field_type,
            label: m.label,
            description: m.description,
            is_localized: m.is_localized,
            is_required: m.is_required,
            default_value: m.default_value,
            validation: m.validation,
            position: m.position,
            is_active: m.is_active,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ── Inputs ───────────────────────────────────────────────────────────────────

/// Input for `createFieldDefinition`.
#[derive(Debug, Clone, InputObject)]
pub struct CreateFieldDefinitionInput {
    /// Target entity type, e.g. "user", "product".
    /// Optional for backward-compatibility (`"user"` is used when omitted).
    pub entity_type: Option<String>,
    pub field_key: String,
    /// Serialised field type, e.g. `"text"`, `"select"`, `"integer"`.
    pub field_type: String,
    /// Localised labels JSON: `{"en": "Phone"}`.
    pub label: JsonValue,
    pub description: Option<JsonValue>,
    #[graphql(default)]
    pub is_localized: bool,
    #[graphql(default)]
    pub is_required: bool,
    pub default_value: Option<JsonValue>,
    pub validation: Option<JsonValue>,
    pub position: Option<i32>,
}

/// Input for `updateFieldDefinition`.
#[derive(Debug, Clone, InputObject)]
pub struct UpdateFieldDefinitionInput {
    /// Target entity type, e.g. "user", "product".
    /// Optional for backward-compatibility (`"user"` is used when omitted).
    pub entity_type: Option<String>,
    pub label: Option<JsonValue>,
    pub description: Option<JsonValue>,
    pub is_localized: Option<bool>,
    pub is_required: Option<bool>,
    pub default_value: Option<JsonValue>,
    pub validation: Option<JsonValue>,
    pub position: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteFieldDefinitionPayload {
    pub success: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct FlexSchemaObject {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub fields_config: JsonValue,
    pub settings: JsonValue,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FlexSchemaView> for FlexSchemaObject {
    fn from(view: FlexSchemaView) -> Self {
        Self {
            id: view.id,
            slug: view.slug,
            name: view.name,
            description: view.description,
            fields_config: serde_json::to_value(view.fields_config)
                .unwrap_or_else(|_| JsonValue::Array(Vec::new())),
            settings: view.settings,
            is_active: view.is_active,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct FlexEntryObject {
    pub id: Uuid,
    pub schema_id: Uuid,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub data: JsonValue,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FlexEntryView> for FlexEntryObject {
    fn from(view: FlexEntryView) -> Self {
        Self {
            id: view.id,
            schema_id: view.schema_id,
            entity_type: view.entity_type,
            entity_id: view.entity_id,
            data: view.data,
            status: view.status,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}

#[derive(Debug, Clone, InputObject)]
pub struct CreateFlexSchemaInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub fields_config: JsonValue,
    pub settings: Option<JsonValue>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpdateFlexSchemaInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub fields_config: Option<JsonValue>,
    pub settings: Option<JsonValue>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, InputObject)]
pub struct CreateFlexEntryInput {
    pub schema_id: Uuid,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub data: JsonValue,
    pub status: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpdateFlexEntryInput {
    pub data: Option<JsonValue>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteFlexPayload {
    pub success: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct FieldValidationErrorObject {
    pub field_key: String,
    pub message: String,
    pub error_code: String,
}

impl From<rustok_core::field_schema::FieldValidationError> for FieldValidationErrorObject {
    fn from(err: rustok_core::field_schema::FieldValidationError) -> Self {
        let code_str = match err.error_code {
            rustok_core::field_schema::FieldErrorCode::Required => "required",
            rustok_core::field_schema::FieldErrorCode::InvalidType => "invalid_type",
            rustok_core::field_schema::FieldErrorCode::TooShort => "too_short",
            rustok_core::field_schema::FieldErrorCode::TooLong => "too_long",
            rustok_core::field_schema::FieldErrorCode::BelowMinimum => "below_minimum",
            rustok_core::field_schema::FieldErrorCode::AboveMaximum => "above_maximum",
            rustok_core::field_schema::FieldErrorCode::PatternMismatch => "pattern_mismatch",
            rustok_core::field_schema::FieldErrorCode::InvalidOption => "invalid_option",
            rustok_core::field_schema::FieldErrorCode::InvalidFormat => "invalid_format",
            rustok_core::field_schema::FieldErrorCode::NestingTooDeep => "nesting_too_deep",
        };
        Self {
            field_key: err.field_key,
            message: err.message,
            error_code: code_str.to_string(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EntryDriftDetailObject {
    pub entry_id: Uuid,
    pub errors: Vec<FieldValidationErrorObject>,
}

impl From<EntryDriftDetail> for EntryDriftDetailObject {
    fn from(view: EntryDriftDetail) -> Self {
        Self {
            entry_id: view.entry_id,
            errors: view.errors.into_iter().map(FieldValidationErrorObject::from).collect(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SchemaRetroValidationReportObject {
    pub schema_id: Uuid,
    pub total_entries_checked: i32,
    pub valid_entries_count: i32,
    pub drifted_entries_count: i32,
    pub drift_details: Vec<EntryDriftDetailObject>,
}

impl From<SchemaRetroValidationReport> for SchemaRetroValidationReportObject {
    fn from(view: SchemaRetroValidationReport) -> Self {
        Self {
            schema_id: view.schema_id,
            total_entries_checked: view.total_entries_checked,
            valid_entries_count: view.valid_entries_count,
            drifted_entries_count: view.drifted_entries_count,
            drift_details: view.drift_details.into_iter().map(EntryDriftDetailObject::from).collect(),
        }
    }
}
