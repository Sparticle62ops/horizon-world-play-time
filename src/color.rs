//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.util.Color`.
//!
//! Upstream parses the saved form with the regex `\((\d+), (\d+), (\d+),
//! (\d+)\)` matched whole; the port parses that exact shape by hand so the
//! port carries no third-party deps, matching the other ports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

impl Color {
    pub const fn new(r: i32, g: i32, b: i32, a: i32) -> Self {
        Self { r, g, b, a }
    }

    /// `String.format("(%d, %d, %d, %d)", ...)`.
    pub fn to_save_string(&self) -> String {
        format!("({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    /// Upstream `fromString`: whole-match `(r, g, b, a)`; a negative alpha
    /// becomes `127 - a`, anything unparseable is `None`.
    pub fn from_save_string(string: &str) -> Option<Self> {
        let inner = string.strip_prefix('(')?.strip_suffix(')')?;
        let mut parts = inner.split(", ").map(str::parse::<i32>);
        let r = parts.next()?.ok()?;
        let g = parts.next()?.ok()?;
        let b = parts.next()?.ok()?;
        let a = parts.next()?.ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self::new(r, g, b, if a < 0 { 127 - a } else { a }))
    }

    pub fn from_argb(argb: i32) -> Self {
        Self::new(
            (argb & 0x00FF_0000) >> 16,
            (argb & 0x0000_FF00) >> 8,
            argb & 0x0000_00FF,
            (argb >> 24) & 0xFF,
        )
    }

    pub fn to_argb(&self) -> i32 {
        (self.a << 24) | (self.r << 16) | (self.g << 8) | self.b
    }
}
