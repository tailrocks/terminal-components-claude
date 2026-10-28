use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputFieldId(String);

impl InputFieldId {
    pub fn new(value: &str) -> Result<Self, SchemaError> {
        let mut bytes = value.bytes();
        let Some(first) = bytes.next() else {
            return Err(SchemaError::InvalidFieldId(value.to_owned()));
        };
        if !(first.is_ascii_lowercase() || first.is_ascii_digit())
            || !bytes.all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
        {
            return Err(SchemaError::InvalidFieldId(value.to_owned()));
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSensitivity {
    Public,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputKind {
    Text,
    Choice(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputField {
    id: InputFieldId,
    label: String,
    help: Option<String>,
    kind: InputKind,
    required: bool,
    default: Option<String>,
    sensitivity: InputSensitivity,
}

impl InputField {
    pub fn text(id: &str, label: &str, required: bool) -> Result<Self, SchemaError> {
        Ok(Self {
            id: InputFieldId::new(id)?,
            label: label.to_owned(),
            help: None,
            kind: InputKind::Text,
            required,
            default: None,
            sensitivity: InputSensitivity::Public,
        })
    }

    pub fn choice(
        id: &str,
        label: &str,
        choices: &[&str],
        required: bool,
    ) -> Result<Self, SchemaError> {
        Ok(Self {
            id: InputFieldId::new(id)?,
            label: label.to_owned(),
            help: None,
            kind: InputKind::Choice(choices.iter().map(|choice| (*choice).to_owned()).collect()),
            required,
            default: None,
            sensitivity: InputSensitivity::Public,
        })
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_owned());
        self
    }

    pub fn with_default(mut self, value: &str) -> Self {
        self.default = Some(value.to_owned());
        self
    }

    pub fn with_sensitivity(mut self, sensitivity: InputSensitivity) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn id(&self) -> &InputFieldId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn default_value(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn choices(&self) -> &[String] {
        match &self.kind {
            InputKind::Text => &[],
            InputKind::Choice(choices) => choices,
        }
    }

    pub fn sensitivity(&self) -> InputSensitivity {
        self.sensitivity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputSchema {
    version: u16,
    fields: Vec<InputField>,
}

impl InputSchema {
    pub fn new(version: u16, fields: Vec<InputField>) -> Result<Self, SchemaError> {
        if version == 0 {
            return Err(SchemaError::ZeroVersion);
        }
        if fields.is_empty() {
            return Err(SchemaError::NoFields);
        }

        let mut seen = BTreeMap::<InputFieldId, ()>::new();
        for field in &fields {
            if field.label.trim().is_empty() {
                return Err(SchemaError::EmptyLabel(field.id.0.clone()));
            }
            if field.sensitivity == InputSensitivity::Secret {
                return Err(SchemaError::SecretInputsUnsupported(field.id.0.clone()));
            }
            if seen.insert(field.id.clone(), ()).is_some() {
                return Err(SchemaError::DuplicateField(field.id.0.clone()));
            }
            if let InputKind::Choice(choices) = &field.kind {
                if choices.is_empty() {
                    return Err(SchemaError::EmptyChoices(field.id.0.clone()));
                }
                let mut seen_choices = BTreeMap::<&str, ()>::new();
                for choice in choices {
                    if choice.trim().is_empty() {
                        return Err(SchemaError::EmptyChoice(field.id.0.clone()));
                    }
                    if seen_choices.insert(choice.as_str(), ()).is_some() {
                        return Err(SchemaError::DuplicateChoice(field.id.0.clone()));
                    }
                }
            }
            if let Some(default) = &field.default {
                if field.required && default.trim().is_empty() {
                    return Err(SchemaError::InvalidDefault(field.id.0.clone()));
                }
                if let InputKind::Choice(choices) = &field.kind
                    && !choices.contains(default)
                {
                    return Err(SchemaError::InvalidDefault(field.id.0.clone()));
                }
            }
        }

        Ok(Self { version, fields })
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn fields(&self) -> &[InputField] {
        &self.fields
    }

    pub fn field(&self, id: &str) -> Option<&InputField> {
        self.fields.iter().find(|field| field.id.as_str() == id)
    }

    pub fn accept(&self, values: &[(&str, &str)]) -> Result<AcceptedInputs, InputError> {
        let mut supplied = BTreeMap::<InputFieldId, String>::new();
        for (raw_id, value) in values {
            let id = InputFieldId::new(raw_id)
                .map_err(|_| InputError::InvalidFieldId((*raw_id).to_owned()))?;
            if self.field(id.as_str()).is_none() {
                return Err(InputError::UnknownField(id.0));
            }
            if supplied.insert(id.clone(), (*value).to_owned()).is_some() {
                return Err(InputError::DuplicateField(id.0));
            }
        }

        let mut accepted = BTreeMap::new();
        for field in &self.fields {
            let value = supplied.remove(&field.id).or_else(|| field.default.clone());
            let Some(value) = value else {
                if field.required {
                    return Err(InputError::MissingRequired(field.id.0.clone()));
                }
                continue;
            };
            if field.required && value.trim().is_empty() {
                return Err(InputError::MissingRequired(field.id.0.clone()));
            }
            if let InputKind::Choice(choices) = &field.kind
                && !choices.contains(&value)
            {
                return Err(InputError::InvalidChoice(field.id.0.clone()));
            }
            accepted.insert(field.id.clone(), value);
        }

        Ok(AcceptedInputs {
            schema: self.clone(),
            values: accepted,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedInputs {
    schema: InputSchema,
    values: BTreeMap<InputFieldId, String>,
}

impl AcceptedInputs {
    pub fn schema(&self) -> &InputSchema {
        &self.schema
    }

    pub fn schema_version(&self) -> u16 {
        self.schema.version
    }

    pub fn value(&self, id: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(field_id, _)| field_id.as_str() == id)
            .map(|(_, value)| value.as_str())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.values.keys().any(|field_id| field_id.as_str() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    ZeroVersion,
    NoFields,
    InvalidFieldId(String),
    EmptyLabel(String),
    DuplicateField(String),
    EmptyChoices(String),
    EmptyChoice(String),
    DuplicateChoice(String),
    InvalidDefault(String),
    SecretInputsUnsupported(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroVersion => f.write_str("input schema version must be non-zero"),
            Self::NoFields => f.write_str("input schema must declare at least one field"),
            Self::InvalidFieldId(id) => write!(f, "invalid input field id: {id}"),
            Self::EmptyLabel(id) => write!(f, "input field {id} has no label"),
            Self::DuplicateField(id) => write!(f, "input field {id} is declared more than once"),
            Self::EmptyChoices(id) => write!(f, "choice field {id} has no choices"),
            Self::EmptyChoice(id) => write!(f, "choice field {id} has an empty choice"),
            Self::DuplicateChoice(id) => write!(f, "choice field {id} repeats a choice"),
            Self::InvalidDefault(id) => write!(f, "input field {id} has an invalid default"),
            Self::SecretInputsUnsupported(id) => {
                write!(f, "secret input field {id} is not supported")
            }
        }
    }
}

impl std::error::Error for SchemaError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    InvalidFieldId(String),
    UnknownField(String),
    DuplicateField(String),
    MissingRequired(String),
    InvalidChoice(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFieldId(id) => write!(f, "invalid input field id: {id}"),
            Self::UnknownField(id) => write!(f, "unknown input field: {id}"),
            Self::DuplicateField(id) => write!(f, "input field {id} was supplied more than once"),
            Self::MissingRequired(id) => write!(f, "required input field {id} is empty"),
            Self::InvalidChoice(id) => write!(f, "input field {id} has an unsupported choice"),
        }
    }
}

impl std::error::Error for InputError {}
