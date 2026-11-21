//! 1:1 ports of upstream's render-position enums. `from_name` is the
//! `EnumValue` read (case-insensitive over the constant names), `to_json`
//! the `EnumValue` write (lowercase name).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerEntryRenderPos {
    AfterName,
    BehindCount,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldEntryRenderPos {
    TopRight,
    Left,
    Right,
}

impl ServerEntryRenderPos {
    pub const DEFAULT: Self = Self::AfterName;

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "after_name" => Some(Self::AfterName),
            "behind_count" => Some(Self::BehindCount),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    pub fn to_json(self) -> String {
        match self {
            Self::AfterName => "after_name".into(),
            Self::BehindCount => "behind_count".into(),
            Self::Left => "left".into(),
            Self::Right => "right".into(),
        }
    }
}

impl WorldEntryRenderPos {
    pub const DEFAULT: Self = Self::TopRight;

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "top_right" => Some(Self::TopRight),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    pub fn to_json(self) -> String {
        match self {
            Self::TopRight => "top_right".into(),
            Self::Left => "left".into(),
            Self::Right => "right".into(),
        }
    }
}
