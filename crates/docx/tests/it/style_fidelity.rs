//! Independently authored minimal regressions for Stage 3 style preservation.
use std::io::{Cursor, Read, Write};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn package(styles: &str, content: &str) -> Vec<u8> {
    let document = format!(r#"<w:document xmlns:w="{W}"><w:body><w:p>{content}</w:p></w:body></w:document>"#);
    let styles = format!(r#"<w:styles xmlns:w="{W}">{styles}</w:styles>"#);
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="styles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="theme" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/></Relationships>"#;
    let theme = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Synthetic"><a:themeElements><a:clrScheme name="Synthetic"><a:dk1><a:srgbClr val="182838"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="203040"/></a:dk2><a:lt2><a:srgbClr val="EEEEEE"/></a:lt2><a:accent1><a:srgbClr val="306090"/></a:accent1></a:clrScheme><a:fontScheme name="Synthetic"><a:majorFont><a:latin typeface="Fixture Heading"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Fixture Body"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme></a:themeElements></a:theme>"#;
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, xml) in [
        ("word/document.xml", document.as_str()),
        ("word/styles.xml", styles.as_str()),
        ("word/_rels/document.xml.rels", rels),
        ("word/theme/theme1.xml", theme),
    ] {
        z.start_file(path, zip::write::SimpleFileOptions::default()).unwrap();
        z.write_all(xml.as_bytes()).unwrap();
    }
    z.finish().unwrap().into_inner()
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut s = String::new();
    z.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}

fn style<'a>(xml: &'a str, id: &str) -> &'a str {
    xml.split("<w:style ")
        .skip(1)
        .find(|s| s.split('>').next().unwrap().contains(&format!("w:styleId=\"{id}\"")))
        .unwrap()
        .split("</w:style>")
        .next()
        .unwrap()
}

#[test]
fn style_fidelity_gallery_flags_are_independent() {
    for flags in ["", "<w:semiHidden/>", "<w:unhideWhenUsed/>", "<w:semiHidden/><w:unhideWhenUsed w:val=\"0\"/>"] {
        let mut bytes =
            package(&format!(r#"<w:style w:type="paragraph" w:styleId="Gallery"><w:name w:val="Gallery"/>{flags}<w:qFormat/></w:style>"#), "");
        for _ in 0..3 {
            let doc = wordcraft_docx::read(&bytes).unwrap();
            bytes = wordcraft_docx::write(&doc).unwrap();
            let xml = part(&bytes, "word/styles.xml");
            let s = style(&xml, "Gallery");
            assert_eq!(s.contains("<w:unhideWhenUsed"), flags.contains("<w:unhideWhenUsed"), "{s}");
            if flags.contains("w:val=\"0\"") {
                assert!(s.contains("<w:unhideWhenUsed w:val=\"0\""), "{s}");
            }
        }
    }
}

#[test]
fn style_fidelity_level_only_numbering_survives_without_creating_a_list() {
    let mut bytes = package(
        r#"<w:style w:type="paragraph" w:styleId="Subtitle"><w:name w:val="Subtitle"/><w:pPr><w:numPr><w:ilvl w:val="1"/></w:numPr></w:pPr></w:style>"#,
        "",
    );
    for _ in 0..3 {
        let doc = wordcraft_docx::read(&bytes).unwrap();
        assert!(doc.styles.get("Subtitle").unwrap().para.numbering.is_none());
        bytes = wordcraft_docx::write(&doc).unwrap();
        let xml = part(&bytes, "word/styles.xml");
        let s = style(&xml, "Subtitle");
        assert!(s.contains("<w:ilvl w:val=\"1\""), "{s}");
        assert!(!s.contains("<w:numId"), "{s}");
    }
}

#[test]
fn style_fidelity_latin_theme_fonts_keep_independent_slots_and_fallbacks() {
    let fonts = r#"<w:rFonts w:ascii="ASCII fallback" w:hAnsi="High fallback" w:asciiTheme="minorHAnsi" w:hAnsiTheme="majorHAnsi"/>"#;
    let mut bytes = package(
        &format!(r#"<w:style w:type="paragraph" w:styleId="Themed"><w:name w:val="Themed"/><w:rPr>{fonts}</w:rPr></w:style>"#),
        &format!("<w:r><w:rPr>{fonts}</w:rPr><w:t>Text</w:t></w:r>"),
    );
    for _ in 0..3 {
        let doc = wordcraft_docx::read(&bytes).unwrap();
        assert_eq!(doc.styles.get("Themed").unwrap().chr.font.as_deref(), Some("Fixture Body"));
        bytes = wordcraft_docx::write(&doc).unwrap();
        for name in ["word/document.xml", "word/styles.xml"] {
            let xml = part(&bytes, name);
            for a in ["w:ascii=\"ASCII fallback\"", "w:hAnsi=\"High fallback\"", "w:asciiTheme=\"minorHAnsi\"", "w:hAnsiTheme=\"majorHAnsi\""] {
                assert!(xml.contains(a), "missing {a}: {xml}");
            }
        }
    }
}

#[test]
fn style_fidelity_theme_text_and_border_colors_keep_their_references() {
    let mut bytes = package(
        r#"<w:style w:type="paragraph" w:styleId="Colors"><w:name w:val="Colors"/><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="8" w:color="123456" w:themeColor="text1" w:themeTint="A6"/></w:pBdr></w:pPr><w:rPr><w:color w:val="234567" w:themeColor="accent1" w:themeShade="BF"/></w:rPr></w:style>"#,
        "",
    );
    for _ in 0..3 {
        let doc = wordcraft_docx::read(&bytes).unwrap();
        bytes = wordcraft_docx::write(&doc).unwrap();
        let xml = part(&bytes, "word/styles.xml");
        for a in [
            "w:themeColor=\"text1\"",
            "w:themeTint=\"A6\"",
            "w:color=\"123456\"",
            "w:themeColor=\"accent1\"",
            "w:themeShade=\"BF\"",
            "w:val=\"234567\"",
        ] {
            assert!(xml.contains(a), "missing {a}: {xml}");
        }
    }
}

#[test]
fn style_fidelity_explicit_same_value_edits_replace_theme_provenance() {
    let bytes = package(
        r#"<w:style w:type="paragraph" w:styleId="Edit"><w:name w:val="Edit"/><w:pPr><w:pBdr><w:bottom w:val="single" w:color="000000" w:themeColor="accent1"/></w:pBdr></w:pPr><w:rPr><w:rFonts w:asciiTheme="minorHAnsi"/><w:color w:val="000000" w:themeColor="accent1"/></w:rPr></w:style>"#,
        "",
    );
    let mut doc = wordcraft_docx::read(&bytes).unwrap();
    let s = std::sync::Arc::make_mut(&mut doc.styles).get_mut("Edit").unwrap();
    s.chr.set_font(s.chr.font.clone());
    s.chr.set_color(s.chr.color);
    let b = s.para.borders.as_mut().unwrap().bottom.as_mut().unwrap();
    b.set_color(b.color);
    let saved = wordcraft_docx::write(&doc).unwrap();
    let xml = part(&saved, "word/styles.xml");
    let s = style(&xml, "Edit");
    assert!(!s.contains("Theme=") && !s.contains("themeColor="), "{s}");
    assert!(s.contains("w:ascii=\"Fixture Body\""));
}

#[test]
fn style_fidelity_overlay_subtraction_and_backwards_json() {
    use wordcraft_doc::props::CharProps;
    let bytes = package(
        r#"<w:style w:type="paragraph" w:styleId="Base"><w:name w:val="Base"/><w:rPr><w:rFonts w:asciiTheme="minorHAnsi"/><w:color w:themeColor="accent1"/></w:rPr></w:style>"#,
        "",
    );
    let doc = wordcraft_docx::read(&bytes).unwrap();
    let base = doc.styles.get("Base").unwrap().chr.clone();
    assert!(base.font_source.is_some() && base.color_source.is_some());
    assert!(base.clone().minus(&base).is_empty());
    let explicit = CharProps { font: base.font.clone(), color: base.color, ..Default::default() };
    let merged = base.clone().overlaid(&explicit);
    assert!(merged.font_source.is_none() && merged.color_source.is_none());
    let difference = explicit.clone().minus(&base);
    assert_eq!(difference.font, explicit.font);
    assert_eq!(difference.color, explicit.color);
}

#[test]
fn style_fidelity_level_only_inheritance_defaults_and_explicit_clear() {
    let bytes = package(
        r#"<w:docDefaults><w:pPrDefault><w:pPr><w:numPr><w:ilvl w:val="3"/></w:numPr></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:styleId="Base"><w:name w:val="Base"/><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="7"/></w:numPr></w:pPr></w:style><w:style w:type="paragraph" w:styleId="Child"><w:name w:val="Child"/><w:basedOn w:val="Base"/><w:pPr><w:numPr><w:ilvl w:val="2"/></w:numPr></w:pPr></w:style>"#,
        "",
    );
    let mut doc = wordcraft_docx::read(&bytes).unwrap();
    assert_eq!(doc.styles.get("Child").unwrap().para.numbering.unwrap().num, 7);
    let saved = wordcraft_docx::write(&doc).unwrap();
    let xml = part(&saved, "word/styles.xml");
    assert!(!style(&xml, "Child").contains("<w:numId"));
    assert!(xml.split("</w:docDefaults>").next().unwrap().contains("<w:ilvl w:val=\"3\""));
    let child = std::sync::Arc::make_mut(&mut doc.styles).get_mut("Child").unwrap();
    child.set_numbering(Some(wordcraft_doc::props::NumRef { num: 0, level: 0 }));
    let saved = wordcraft_docx::write(&doc).unwrap();
    assert!(style(&part(&saved, "word/styles.xml"), "Child").contains("<w:numId w:val=\"0\""));
}

#[test]
fn style_fidelity_reference_only_fonts_and_invalid_color_references_are_bounded() {
    let bytes = package(
        r#"<w:style w:type="character" w:styleId="Accent"><w:name w:val="Accent"/><w:rPr><w:rFonts w:asciiTheme="majorHAnsi" w:hAnsiTheme="minorHAnsi"/><w:color w:val="123456" w:themeColor="unknown" w:themeTint="XX"/></w:rPr></w:style>"#,
        "",
    );
    let doc = wordcraft_docx::read(&bytes).unwrap();
    let saved = wordcraft_docx::write(&doc).unwrap();
    let xml = part(&saved, "word/styles.xml");
    let s = style(&xml, "Accent");
    assert!(s.contains("w:asciiTheme=\"majorHAnsi\""));
    assert!(!s.contains("w:ascii=\""));
    assert!(!s.contains("themeColor=\"unknown\""));
    assert!(s.contains("w:val=\"123456\""));
}
