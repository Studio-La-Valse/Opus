#[cfg(test)]
mod tests {
    use lib::score::layout_options::UserLayout;
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

    /// Every `group.field` path on `UserLayout`, recovered by serializing a
    /// default instance rather than from a second hand-maintained list.
    fn user_layout_paths() -> BTreeSet<String> {
        let value = serde_json::to_value(UserLayout::default())
            .expect("UserLayout must serialize to enumerate its fields");
        let groups = value
            .as_object()
            .expect("UserLayout serializes as a JSON object");

        groups
            .iter()
            .flat_map(|(group, fields)| {
                fields
                    .as_object()
                    .unwrap_or_else(|| panic!("group '{group}' serializes as a JSON object"))
                    .keys()
                    .map(move |field| format!("{group}.{field}"))
            })
            .collect()
    }

    /// A value each option's Rust type will accept, keyed by path since the
    /// colours, enums and fonts need real spellings; everything else on
    /// `UserLayout` is a plain number.
    fn plausible_value(path: &str) -> Value {
        match path {
            "page.color" | "foreground.color" => Value::String("#112233".to_string()),
            "section.symbol" | "part_group.symbol" | "part.symbol" => {
                Value::String("bracket".to_string())
            }
            "group_name.font" | "title.font" | "lyric.font" => Value::String("serif".to_string()),
            _ => Value::from(1.5),
        }
    }

    /// Guards the correspondence between `LAYOUT_OPTIONS` in
    /// `web/music-xml.js` and the options of `UserLayout` in both directions.
    /// A JS path with no matching option - the `"staff"` typo that let
    /// `--staff-line-width` silently do nothing - fails to deserialize
    /// thanks to `deny_unknown_fields`. A `UserLayout` option missing from the
    /// JS list fails the explicit set comparison below.
    #[test]
    fn layout_options_matches_user_layout_option_for_option() {
        let js_paths = layout_options_from_js();
        assert!(
            !js_paths.is_empty(),
            "failed to parse any paths out of LAYOUT_OPTIONS in web/music-xml.js"
        );

        let mut object = serde_json::Map::new();
        for path in &js_paths {
            let (group, field) = path
                .split_once('.')
                .unwrap_or_else(|| panic!("LAYOUT_OPTIONS entry '{path}' is not group.field"));
            object
                .entry(group)
                .or_insert_with(|| Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .expect("a group is an object")
                .insert(field.to_string(), plausible_value(path));
        }

        serde_json::from_value::<UserLayout>(Value::Object(object)).unwrap_or_else(|e| {
            panic!("every LAYOUT_OPTIONS path must deserialize as a UserLayout option: {e}")
        });

        let js_paths: BTreeSet<String> = js_paths.into_iter().collect();
        assert_eq!(
            js_paths,
            user_layout_paths(),
            "LAYOUT_OPTIONS in web/music-xml.js must list exactly the UserLayout options"
        );
    }
}
