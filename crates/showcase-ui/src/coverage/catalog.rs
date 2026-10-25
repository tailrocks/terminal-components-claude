//! Typed entries for the source scanner's declared syntax scope.

/// Item kinds emitted by the bounded source scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapKind {
    /// A public struct or enum.
    Type,
    /// A variant of a public enum.
    Variant,
    /// A public method taking `self` and returning `Self`.
    Builder,
    /// A public field of one of the selected token structs.
    Token,
    /// A public free function or method.
    Function,
}

impl CapKind {
    /// Name printed in drift diagnostics.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Variant => "variant",
            Self::Builder => "builder",
            Self::Token => "token",
            Self::Function => "function",
        }
    }

    /// Constructor name used by the generated inventory.
    pub const fn ctor(self) -> &'static str {
        match self {
            Self::Type => "ty",
            Self::Variant => "variant",
            Self::Builder => "builder",
            Self::Token => "token",
            Self::Function => "function",
        }
    }
}

/// One item discovered within the scanner's declared scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability {
    /// Stable item ID, `crate::module::Name`.
    pub id: &'static str,
    /// Scanner item kind.
    pub kind: CapKind,
}

impl Capability {
    /// A public struct or enum.
    pub const fn ty(id: &'static str) -> Self {
        Self {
            id,
            kind: CapKind::Type,
        }
    }
    /// A public enum variant.
    pub const fn variant(id: &'static str) -> Self {
        Self {
            id,
            kind: CapKind::Variant,
        }
    }
    /// A builder method.
    pub const fn builder(id: &'static str) -> Self {
        Self {
            id,
            kind: CapKind::Builder,
        }
    }
    /// A selected public token field.
    pub const fn token(id: &'static str) -> Self {
        Self {
            id,
            kind: CapKind::Token,
        }
    }
    /// A public function or method.
    pub const fn function(id: &'static str) -> Self {
        Self {
            id,
            kind: CapKind::Function,
        }
    }
}

/// Finds one inventory item by ID.
pub fn find(id: &str) -> Option<&'static Capability> {
    CAPABILITIES.iter().find(|cap| cap.id == id)
}

#[path = "catalog_data.rs"]
mod catalog_data;

/// Snapshot of the items found within the scanner's declared scope.
pub use catalog_data::CAPABILITIES;
