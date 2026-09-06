#[cfg(test)]
mod tests {
    use cli::commands::read_musicxml;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const DOC: &str = "<?xml version=\"1.0\"?>\n<score-partwise version=\"4.0\"/>\n";

    fn scratch_file(tag: &str, bytes: &[u8]) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("opus-musicxml-source-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("score.musicxml");
        fs::write(&path, bytes).unwrap();
        path
    }

    fn read(tag: &str, bytes: &[u8]) -> String {
        let path = scratch_file(tag, bytes);
        read_musicxml(path.to_str().unwrap())
    }

    fn utf16(doc: &str, big_endian: bool) -> Vec<u8> {
        let mut out = if big_endian {
            vec![0xFE, 0xFF]
        } else {
            vec![0xFF, 0xFE]
        };

        for unit in doc.encode_utf16() {
            let pair = if big_endian {
                unit.to_be_bytes()
            } else {
                unit.to_le_bytes()
            };
            out.extend_from_slice(&pair);
        }

        out
    }

    #[test]
    fn plain_utf8_is_read_unchanged() {
        assert_eq!(read("utf8", DOC.as_bytes()), DOC);
    }

    /// A UTF-8 BOM is a byte-order mark, not document content: leaving it in
    /// puts a U+FEFF in front of the XML declaration, which is not a thing an
    /// XML parser has to accept.
    #[test]
    fn a_utf8_bom_is_stripped() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(DOC.as_bytes());

        assert_eq!(read("utf8-bom", &bytes), DOC);
    }

    #[test]
    fn utf16_is_decoded_in_both_byte_orders() {
        assert_eq!(read("utf16-le", &utf16(DOC, false)), DOC);
        assert_eq!(read("utf16-be", &utf16(DOC, true)), DOC);
    }

    /// Non-ASCII has to survive the transcode, since that is most of the point
    /// of a document being in UTF-16 in the first place.
    #[test]
    fn utf16_round_trips_non_ascii_content() {
        let doc = "<work-title>Rêve · 夢</work-title>";

        assert_eq!(read("utf16-wide-le", &utf16(doc, false)), doc);
        assert_eq!(read("utf16-wide-be", &utf16(doc, true)), doc);
    }

    /// The two Mozart samples are genuine UTF-16 exports (BOM plus
    /// `encoding="UTF-16"`), which is what motivated decoding by BOM rather
    /// than handing the bytes straight to `String::from_utf8`.
    #[test]
    fn the_bundled_utf16_samples_decode_to_xml() {
        for name in ["MozaChloSample", "MozaVeilSample"] {
            let path = format!(
                "{}/../assets/xmlsamples/{name}.musicxml",
                env!("CARGO_MANIFEST_DIR")
            );
            let text = read_musicxml(&path);

            assert!(
                text.starts_with("<?xml"),
                "{name} did not decode to an XML declaration, got {:?}",
                &text[..text.len().min(40)]
            );
            assert!(text.contains("<score-partwise"), "{name} is not partwise");
        }
    }
}
