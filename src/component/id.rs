use crate::component::{SourceInfo, Target};
use base64::{engine, Engine};
use std::hash::{DefaultHasher, Hasher};

/// Represents a component identifier, including its display name, symbol name, local file name, hash, and optional scope.
///
/// This information is used to uniquely identify a component in the Qwik framework.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    pub display_name: String,
    pub symbol_name: String,
    pub local_file_name: String,
    pub hash: String,
    pub scope: Option<String>,
}

impl Id {
    fn sanitize(input: &str) -> String {
        input
            .chars()
            .fold((String::new(), false), |(mut acc, uscore), c| {
                if c.is_ascii_alphanumeric() {
                    acc.push(c);
                    (acc, false)
                } else if uscore {
                    // Never push consecutive underscores.
                    (acc, true)
                } else {
                    acc.push('_');
                    (acc, true)
                }
            })
            .0
    }

    fn calculate_hash(local_file_name: &str, display_name: &str, scope: &Option<String>) -> String {
        let mut hasher = DefaultHasher::new();
        if let Some(scope) = scope {
            hasher.write(scope.as_bytes());
        }
        hasher.write(local_file_name.as_bytes());
        hasher.write(display_name.as_bytes());
        let hash = hasher.finish();
        engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hash.to_le_bytes())
            .replace(['-', '_'], "0")
    }

    /// Creates a component [Id] from a given [SourceInfo], a `Vec[String]` of segment identifiers that relate back the
    /// components location in the source code, a target (prod, lib, dev, test), and an optional scope.
    ///
    /// The [Id] contains enough information to uniquely identify a component.
    ///
    /// # Segments
    ///
    /// Segments represent an order list of identifiers that uniquely reference a component in the source code.
    ///
    /// ## Example
    ///
    /// ```javascript
    /// export const Counter = component$(() => {
    ///   const store = useStore({ count: 0 });
    ///   return (
    ///     <>
    ///       I am a dynamic component. Qwik will download me only when it is time to re-render me after the
    ///       user clicks on the <code>+1</code> button.
    ///       <br />
    ///       Current count: {store.count}
    ///       <br />
    ///       <button onClick$={() => store.count++}>+1</button>
    ///     </>
    ///   );
    /// });
    /// ```
    /// For this example, the segments that would be provided to [SourceInfo::new] would be: [Counter, component, button, onClick].
    ///
    /// # Target
    ///
    /// The provide [Target] will determine how the [`Id.symbol_name`](field@Id::symbol_name) is generated.
    ///
    /// When [Target::Dev] or [Target::Test] is provided, the symbol name will be generated as `{display_name}_{hash}`.
    ///
    /// ## Examples
    ///
    /// If display_name is `a_b_c` and the hash is `0RVAWYCCxyk`, the symbol name will be `a_b_c_0RVAWYCCxyk`.
    ///
    /// When [Target::Lib] or [Target::Prod] is provided, the symbol name will be generated as `s_{hash}`.
    ///
    /// ## Examples
    ///
    /// If display_name is `a_b_c` and the hash is `0RVAWYCCxyk`, the symbol name will be `s_0RVAWYCCxyk`.
    ///
    ///
    /// # Hash Generation Semantics
    ///
    /// The hash is generated by creating a `DefaultHasher` and writing the following values, converted to bytes, to it:
    /// - The calculated `display_name`
    /// - The [`SourceInfo::rel_path`](field@SourceInfo::rel_path)
    /// - The `scope` (if provided).
    ///
    /// [V 1.0 REF] see `QwikTransform.register_context_name` in `transform.rs.
    pub fn new(
        source_info: &SourceInfo,
        segments: &Vec<String>,
        target: &Target,
        scope: &Option<String>,
    ) -> Id {
        let local_file_name = source_info.rel_path.to_string_lossy();

        let mut display_name = String::new();

        for segment in segments {
            if display_name.is_empty()
                && segment
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                display_name = format!("_{}", segment);
            } else {
                let prefix: String = if display_name.is_empty() {
                    "".to_string()
                } else {
                    format!("{}_", display_name).to_string()
                };
                display_name = format!("{}{}", prefix, segment);
            }
        }
        display_name = Self::sanitize(&display_name);

        let normalized_local_file_name = local_file_name
            .strip_prefix("./")
            .unwrap_or(&local_file_name);
        let hash64 = Self::calculate_hash(normalized_local_file_name, &display_name, scope);

        let symbol_name = match target {
            Target::Dev | Target::Test => format!("{}_{}", display_name, hash64),
            Target::Lib | Target::Prod => format!("s_{}", hash64),
        };

        let display_name = format!("{}_{}", &source_info.file_name, display_name);

        let local_file_name = format!("{}_{}", local_file_name, symbol_name);
        Id {
            display_name,
            symbol_name,
            local_file_name,
            hash: hash64,
            scope: scope.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_a_name() {
        let name0 = Id::sanitize("a'b-c");
        let name1 = Id::sanitize("A123b_c-~45");
        assert_eq!(name0, "a_b_c");
        assert_eq!(name1, "A123b_c_45");
    }

    #[test]
    fn test_calculate_hash() {
        let hash0 = Id::calculate_hash("./app.js", "a_b_c", &None);
        let hash1 = Id::calculate_hash("./app.js", "a_b_c", &Some("scope".to_string()));
        assert_eq!(hash0, "0RVAWYCCxyk");
        assert_ne!(hash1, hash0);
    }

    #[test]
    fn creates_a_id() {
        let source_info0 = SourceInfo::new("app.js").unwrap();
        let id0 = Id::new(
            &source_info0,
            &vec!["a".to_string(), "b".to_string(), "c".to_string()],
            &Target::Dev,
            &Option::None,
        );
        let hash0 = Id::calculate_hash("app.js", "a_b_c", &None);

        let expected0 = Id {
            display_name: "app.js_a_b_c".to_string(),
            symbol_name: format!("a_b_c_{}", hash0),
            local_file_name: "app.js_a_b_c_tZuivXMgs2w".to_string(),
            hash: hash0,
            scope: None,
        };

        let scope1 = Some("scope".to_string());
        let id1 = Id::new(
            &source_info0,
            &vec!["1".to_string(), "b".to_string(), "c".to_string()],
            &Target::Prod,
            &scope1,
        );
        // Leading  segments that are digits are prefixed with an additional underscore.
        let hash1 = Id::calculate_hash("app.js", "_1_b_c", &scope1);
        let expected1 = Id {
            display_name: "app.js__1_b_c".to_string(),
            // When Target is neither "Dev" nor "Test", the symbol name is set to "s_{hash}".
            symbol_name: format!("s_{}", hash1),
            local_file_name: "app.js_s_bQ4D62Vr0Zg".to_string(),
            hash: hash1,
            scope: Some("scope".to_string()),
        };

        assert_eq!(id0, expected0);
        assert_eq!(id1, expected1);
    }
}
