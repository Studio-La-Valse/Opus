#[cfg(test)]
mod tests {
    use lib::score::user_layout::UserLayout;
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::fs::read_to_string;

    /// Pulls the plain string literals out of `LAYOUT_OPTIONS` in
    /// `web/music-xml.js` by locating the array's brackets and parsing its
    /// lines - good enough for an array of one-per-line quoted names, and
    /// avoids a JS parser dependency for a one-shot extraction.
    fn layout_options_from_js() -> Vec<String> {
        let js = read_to_string(format!(
            "{}/../web/music-xml.js",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("failed to read web/music-xml.js");

        let decl_start = js
            .find("const LAYOUT_OPTIONS = [")
            .expect("LAYOUT_OPTIONS declaration not found in web/music-xml.js");
        let body_start = decl_start + js[decl_start..].find('[').unwrap() + 1;
        let body_end = body_start
            + js[body_start..]
                .find(']')
                .expect("LAYOUT_OPTIONS array is not closed");
        let body = &js[body_start..body_end];

        body.lines()
            .filter_map(|line| {
                let name = line.trim().trim_end_matches(',').trim_matches('"');
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                }
            })
            .collect()
    }

    /// A value each option's Rust type will accept, keyed by name since the
    /// colours and enums need real spellings; everything else on
    /// `UserLayout` is a plain number.
    fn plausible_value(name: &str) -> Value {
        match name {
            "pageColor" | "foregroundColor" => Value::String("#112233".to_string()),
            "pageOrientation" => Value::String("vertical".to_string()),
            "sectionSymbol" | "partGroupSymbol" | "partSymbol" => {
                Value::String("bracket".to_string())
            }
            _ => Value::from(1.5),
        }
    }

    /// Guards the correspondence between `LAYOUT_OPTIONS` in
    /// `web/music-xml.js` and the fields of `UserLayout` in both directions.
    /// A JS name with no matching field - the `"staff"` typo that let
    /// `--staff-line-width` silently do nothing - fails to deserialize
    /// thanks to `deny_unknown_fields`. A `UserLayout` field missing from the
    /// JS list fails the explicit set comparison below, since `UserLayout`'s
    /// own field names are recovered by serializing a default instance
    /// rather than a second hand-maintained list in this test.
    #[test]
    fn layout_options_matches_user_layout_field_for_field() {
        let js_names = layout_options_from_js();
        assert!(
            !js_names.is_empty(),
            "failed to parse any names out of LAYOUT_OPTIONS in web/music-xml.js"
        );

        let object: serde_json::Map<String, Value> = js_names
            .iter()
            .map(|name| (name.clone(), plausible_value(name)))
            .collect();

        serde_json::from_value::<UserLayout>(Value::Object(object)).unwrap_or_else(|e| {
            panic!("every LAYOUT_OPTIONS name must deserialize as a UserLayout field: {e}")
        });

        let js_names: BTreeSet<String> = js_names.into_iter().collect();
        let rust_names: BTreeSet<String> = serde_json::to_value(UserLayout::default())
            .expect("UserLayout must serialize to enumerate its fields")
            .as_object()
            .expect("UserLayout serializes as a JSON object")
            .keys()
            .cloned()
            .collect();

        assert_eq!(
            js_names, rust_names,
            "LAYOUT_OPTIONS in web/music-xml.js must list exactly the UserLayout fields"
        );
    }
}
