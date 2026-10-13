use facet_core::{Def, Facet, Type, UserType};
use facet_reflect::{Peek, ValueId};

/// Compare the complete reflected state, without requiring root `PartialEq`.
/// `None` means equality, `Some` identifies a mismatch, and errors identify
/// values whose comparison cannot be established safely through reflection.
pub(crate) fn compare_arbitrary_values<T: Facet<'static>>(
    original: &T,
    parsed: &T,
) -> Result<Option<String>, String> {
    compare_at(Peek::new(original), Peek::new(parsed), "$", &mut Vec::new())
}

fn compare_at(
    original: Peek<'_, '_>,
    parsed: Peek<'_, '_>,
    path: &str,
    active: &mut Vec<(ValueId, ValueId)>,
) -> Result<Option<String>, String> {
    if original.shape() != parsed.shape() {
        return Err(format!(
            "cannot compare {path}: different reflected types {} and {}",
            original.shape(),
            parsed.shape()
        ));
    }
    let pair = (original.id(), parsed.id());
    if active.contains(&pair) {
        return Err(format!(
            "cannot compare {path} ({}): cyclic reflected value",
            original.shape()
        ));
    }
    active.push(pair);
    let result = compare_contents(original, parsed, path, active);
    active.pop();
    result
}

fn compare_contents(
    original: Peek<'_, '_>,
    parsed: Peek<'_, '_>,
    path: &str,
    active: &mut Vec<(ValueId, ValueId)>,
) -> Result<Option<String>, String> {
    let shape = original.shape();
    // Semantic container definitions take precedence over their Rust layouts.
    // Never invoke parent struct/enum equality: it may intentionally omit fields.
    match shape.def {
        Def::Option(_) => {
            let original = original
                .into_option()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_option()
                .map_err(|error| failed(path, shape, error))?;
            match (original.value(), parsed.value()) {
                (Some(original), Some(parsed)) => {
                    compare_at(original, parsed, &format!("{path}.Some"), active)
                }
                (None, None) => Ok(None),
                _ => Ok(different(path, shape, "Some/None differ")),
            }
        }
        Def::Result(_) => {
            let original = original
                .into_result()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_result()
                .map_err(|error| failed(path, shape, error))?;
            match (original.ok(), parsed.ok(), original.err(), parsed.err()) {
                (Some(original), Some(parsed), _, _) => {
                    compare_at(original, parsed, &format!("{path}.Ok"), active)
                }
                (_, _, Some(original), Some(parsed)) => {
                    compare_at(original, parsed, &format!("{path}.Err"), active)
                }
                _ => Ok(different(path, shape, "Ok/Err differ")),
            }
        }
        Def::List(_) | Def::Array(_) | Def::Slice(_) => {
            let original = original
                .into_list_like()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_list_like()
                .map_err(|error| failed(path, shape, error))?;
            if original.len() != parsed.len() {
                return Ok(different(path, shape, "sequence lengths differ"));
            }
            for (index, (original, parsed)) in original.iter().zip(parsed.iter()).enumerate() {
                if let Some(difference) =
                    compare_at(original, parsed, &format!("{path}[{index}]"), active)?
                {
                    return Ok(Some(difference));
                }
            }
            Ok(None)
        }
        Def::Map(_) => {
            let original = original
                .into_map()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_map()
                .map_err(|error| failed(path, shape, error))?;
            if original.len() != parsed.len() {
                return Ok(different(path, shape, "map lengths differ"));
            }
            let candidates = parsed.iter().collect::<Vec<_>>();
            let mut matched = vec![false; candidates.len()];
            for (index, (key, value)) in original.iter().enumerate() {
                let key_path = format!("{path}[{index}].key");
                let mut found = false;
                for (candidate_index, (candidate_key, candidate_value)) in
                    candidates.iter().enumerate()
                {
                    if matched[candidate_index]
                        || compare_at(key, *candidate_key, &key_path, active)?.is_some()
                    {
                        continue;
                    }
                    if let Some(difference) = compare_at(
                        value,
                        *candidate_value,
                        &format!("{path}[{index}].value"),
                        active,
                    )? {
                        return Ok(Some(difference));
                    }
                    matched[candidate_index] = true;
                    found = true;
                    break;
                }
                if !found {
                    return Ok(different(
                        &key_path,
                        key.shape(),
                        "no matching reflected map key",
                    ));
                }
            }
            Ok(None)
        }
        Def::Set(_) => {
            let original = original
                .into_set()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_set()
                .map_err(|error| failed(path, shape, error))?;
            if original.len() != parsed.len() {
                return Ok(different(path, shape, "set lengths differ"));
            }
            let candidates = parsed.iter().collect::<Vec<_>>();
            let mut matched = vec![false; candidates.len()];
            for (index, value) in original.iter().enumerate() {
                let value_path = format!("{path}[{index}]");
                let mut found = false;
                for (candidate_index, candidate) in candidates.iter().enumerate() {
                    if matched[candidate_index]
                        || compare_at(value, *candidate, &value_path, active)?.is_some()
                    {
                        continue;
                    }
                    matched[candidate_index] = true;
                    found = true;
                    break;
                }
                if !found {
                    return Ok(different(
                        &value_path,
                        value.shape(),
                        "no matching reflected set value",
                    ));
                }
            }
            Ok(None)
        }
        Def::Pointer(_) => {
            let original = original
                .into_pointer()
                .map_err(|error| failed(path, shape, error))?;
            let parsed = parsed
                .into_pointer()
                .map_err(|error| failed(path, shape, error))?;
            match (original.borrow_inner(), parsed.borrow_inner()) {
                (Some(original), Some(parsed)) => {
                    compare_at(original, parsed, &format!("{path}.*"), active)
                }
                _ => Err(failed(path, shape, "pointer contents are not reflectable")),
            }
        }
        Def::Scalar => compare_leaf(original, parsed, path),
        _ => match shape.ty {
            Type::User(UserType::Struct(_)) => {
                let original = original
                    .into_struct()
                    .map_err(|error| failed(path, shape, error))?;
                let parsed = parsed
                    .into_struct()
                    .map_err(|error| failed(path, shape, error))?;
                for (index, field) in original.ty().fields.iter().enumerate() {
                    let field_path = format!("{path}.{}", field.name);
                    let original = original
                        .field(index)
                        .map_err(|error| failed(&field_path, shape, error))?;
                    let parsed = parsed
                        .field(index)
                        .map_err(|error| failed(&field_path, shape, error))?;
                    if let Some(difference) = compare_at(original, parsed, &field_path, active)? {
                        return Ok(Some(difference));
                    }
                }
                Ok(None)
            }
            Type::User(UserType::Enum(_)) => {
                let original = original
                    .into_enum()
                    .map_err(|error| failed(path, shape, error))?;
                let parsed = parsed
                    .into_enum()
                    .map_err(|error| failed(path, shape, error))?;
                let original_variant = original
                    .active_variant()
                    .map_err(|error| failed(path, shape, error))?;
                let parsed_variant = parsed
                    .active_variant()
                    .map_err(|error| failed(path, shape, error))?;
                if original_variant.name != parsed_variant.name {
                    return Ok(different(path, shape, "active enum variants differ"));
                }
                for (index, field) in original_variant.data.fields.iter().enumerate() {
                    let field_path = format!("{path}.{}.{}", original_variant.name, field.name);
                    let original = original
                        .field(index)
                        .map_err(|error| failed(&field_path, shape, error))?
                        .ok_or_else(|| {
                            failed(&field_path, shape, "original variant field is unavailable")
                        })?;
                    let parsed = parsed
                        .field(index)
                        .map_err(|error| failed(&field_path, shape, error))?
                        .ok_or_else(|| {
                            failed(&field_path, shape, "parsed variant field is unavailable")
                        })?;
                    if let Some(difference) = compare_at(original, parsed, &field_path, active)? {
                        return Ok(Some(difference));
                    }
                }
                Ok(None)
            }
            _ => compare_leaf(original, parsed, path),
        },
    }
}

fn compare_leaf(
    original: Peek<'_, '_>,
    parsed: Peek<'_, '_>,
    path: &str,
) -> Result<Option<String>, String> {
    match original.partial_eq(&parsed) {
        Ok(true) => Ok(None),
        Ok(false) => Ok(different(path, original.shape(), "leaf values differ")),
        Err(error) => Err(failed(path, original.shape(), error)),
    }
}

fn different(path: &str, shape: &facet_core::Shape, detail: &str) -> Option<String> {
    Some(format!("{path} ({shape}): {detail}"))
}

fn failed(path: &str, shape: &facet_core::Shape, error: impl std::fmt::Display) -> String {
    format!("cannot compare {path} ({shape}): {error}")
}

#[cfg(test)]
mod tests {
    use super::compare_arbitrary_values;
    use facet::Facet;
    use std::borrow::Cow;
    use std::collections::{HashMap, HashSet};

    // Every aggregate deliberately omits PartialEq.
    #[derive(Clone, Facet)]
    struct Nested {
        value: Option<String>,
        items: Vec<u16>,
    }

    #[allow(
        dead_code,
        reason = "Comparison fixtures are inspected through Facet reflection"
    )]
    #[derive(Clone, Facet)]
    #[repr(u8)]
    enum Command {
        Run { enabled: bool },
        Stop,
    }

    #[derive(Clone, Facet)]
    struct Root {
        nested: Nested,
        command: Command,
        mapping: HashMap<String, Nested>,
        labels: HashSet<String>,
        fixed: [u32; 2],
        boxed: Box<Nested>,
        text: Cow<'static, str>,
    }

    fn fixture(reverse: bool) -> Root {
        let nested = Nested {
            value: Some("kept".into()),
            items: vec![1, 2],
        };
        let mut entries = vec![
            ("first".to_string(), nested.clone()),
            ("second".to_string(), nested.clone()),
        ];
        if reverse {
            entries.reverse();
        }
        Root {
            nested: nested.clone(),
            command: Command::Run { enabled: true },
            mapping: entries.into_iter().collect(),
            labels: ["first".to_string(), "second".to_string()]
                .into_iter()
                .collect(),
            fixed: [3, 4],
            boxed: Box::new(nested),
            text: if reverse {
                Cow::Owned("same content".to_string())
            } else {
                Cow::Borrowed("same content")
            },
        }
    }

    #[test]
    fn compares_nested_state_without_native_equality_or_container_iteration_order() {
        assert_eq!(
            compare_arbitrary_values(&fixture(false), &fixture(true)).unwrap(),
            None
        );
    }

    #[test]
    fn reports_changed_nested_state_and_enum_selection() {
        let original = fixture(false);
        let mut parsed = fixture(true);
        parsed.nested.items[1] = 9;
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(difference.contains("$.nested.items[1]"), "{difference}");
        assert!(difference.contains("u16"), "{difference}");

        let mut parsed = fixture(true);
        parsed.command = Command::Stop;
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(
            difference.contains("$.command") && difference.contains("enum variants"),
            "{difference}"
        );
        parsed.command = Command::Run { enabled: false };
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(difference.contains("$.command.Run.enabled"), "{difference}");
    }

    #[test]
    fn preserves_option_presence_and_finds_changes_in_unordered_values() {
        let original = fixture(false);
        let mut parsed = fixture(true);
        parsed.nested.value = None;
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(
            difference.contains("$.nested.value") && difference.contains("Some/None"),
            "{difference}"
        );

        let mut parsed = fixture(true);
        parsed.mapping.get_mut("second").unwrap().items[0] = 9;
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(
            difference.contains("$.mapping[") && difference.contains(".value.items[0]"),
            "{difference}"
        );

        let mut parsed = fixture(true);
        parsed.labels.remove("second");
        parsed.labels.insert("changed".to_string());
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(
            difference.contains("$.labels[") && difference.contains("set value"),
            "{difference}"
        );
    }

    #[test]
    fn parent_partial_eq_cannot_hide_reflected_fields() {
        #[derive(Facet)]
        struct IncompleteEquality {
            retained: u8,
            omitted: u8,
        }
        impl PartialEq for IncompleteEquality {
            fn eq(&self, other: &Self) -> bool {
                self.retained == other.retained
            }
        }
        let original = IncompleteEquality {
            retained: 1,
            omitted: 2,
        };
        let parsed = IncompleteEquality {
            retained: 1,
            omitted: 3,
        };
        assert!(original == parsed);
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(difference.contains("$.omitted"), "{difference}");
    }

    #[test]
    fn tuples_compare_non_eq_members_and_fields_omitted_by_member_equality() {
        #[derive(Clone, Facet)]
        struct SelectiveEquality {
            retained: u8,
            omitted: u8,
        }
        impl PartialEq for SelectiveEquality {
            fn eq(&self, other: &Self) -> bool {
                self.retained == other.retained
            }
        }

        let original = (
            Nested {
                value: Some("kept".into()),
                items: vec![1, 2],
            },
            SelectiveEquality {
                retained: 3,
                omitted: 4,
            },
        );
        let mut parsed = original.clone();
        assert_eq!(compare_arbitrary_values(&original, &parsed).unwrap(), None);

        parsed.0.items[0] = 9;
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(difference.contains("$.0.items[0]"), "{difference}");

        parsed.0.items[0] = 1;
        parsed.1.omitted = 9;
        assert!(original.1 == parsed.1);
        let difference = compare_arbitrary_values(&original, &parsed)
            .unwrap()
            .unwrap();
        assert!(difference.contains("$.1.omitted"), "{difference}");
    }

    #[test]
    fn opaque_values_without_equality_are_an_explicit_comparison_error() {
        #[allow(
            dead_code,
            reason = "This opaque fixture deliberately has no reflected or native field equality"
        )]
        #[derive(Facet)]
        #[facet(opaque)]
        struct UnsupportedLeaf(u32);
        #[derive(Facet)]
        struct Root {
            unavailable: UnsupportedLeaf,
        }
        let original = Root {
            unavailable: UnsupportedLeaf(1),
        };
        let parsed = Root {
            unavailable: UnsupportedLeaf(1),
        };
        let error = compare_arbitrary_values(&original, &parsed).unwrap_err();
        assert!(
            error.contains("cannot compare $.unavailable") && error.contains("UnsupportedLeaf"),
            "{error}"
        );
    }
}
