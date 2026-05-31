//! Core value types for my-project.
//!
//! [TODO: Replace with your project's actual core types and documentation.]

#![forbid(unsafe_code)]

/// An example value type — replace with your core domain type.
///
/// # Examples
///
/// ```
/// use my_project_core::Example;
///
/// let e = Example { id: 42 };
/// assert_eq!(e.id, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Example {
    /// A numeric identifier.
    pub id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest::proptest! {
        #[test]
        fn example_roundtrip(id: u32) {
            let e = Example { id };
            let json = serde_json::to_string(&e).unwrap();
            let back: Example = serde_json::from_str(&json).unwrap();
            assert_eq!(e, back);
        }
    }
}
