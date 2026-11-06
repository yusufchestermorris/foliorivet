//! Original synthetic preservation regressions for the FolioRivet Stage 2 findings.
use std::io::{Cursor, Read, Write};

fn package(document: &str, styles: &str) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="styles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    for (path, xml) in [("word/document.xml", document), ("word/styles.xml", styles), ("word/_rels/document.xml.rels", rels)] {
        zip.start_file(path, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

#[test]
fn fidelity_partial_table_and_row_margins_survive_repeated_saves() {
    let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{ns}"><w:style w:type="table" w:styleId="Base"><w:name w:val="Base"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="20"/><w:left w:w="108"/><w:bottom w:w="40"/><w:right w:w="108"/></w:tblCellMar></w:tblPr></w:style><w:style w:type="table" w:styleId="Derived"><w:name w:val="Derived"/><w:basedOn w:val="Base"/><w:tblPr><w:tblCellMar><w:start w:w="80"/></w:tblCellMar></w:tblPr></w:style></w:styles>"#
    );
    for (xml, sides) in [
        (r#"<w:top w:w="0"/><w:bottom w:w="60"/>"#, [Some(0.0), None, Some(3.0), None]),
        (r#"<w:start w:w="0"/><w:end w:w="80"/>"#, [None, Some(0.0), None, Some(4.0)]),
        ("", [None; 4]),
        (r#"<w:top w:w="20"/><w:left w:w="0"/><w:bottom w:w="60"/><w:right w:w="80"/>"#, [Some(1.0), Some(0.0), Some(3.0), Some(4.0)]),
    ] {
        let document = format!(
            r#"<w:document xmlns:w="{ns}"><w:body><w:tbl><w:tblPr><w:tblStyle w:val="Derived"/><w:tblCellMar><w:top w:w="100"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tblPrEx><w:tblCellMar>{xml}</w:tblCellMar></w:tblPrEx><w:tc><w:p><w:r><w:t>Padding</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#
        );
        let mut bytes = package(&document, &styles);
        for _ in 0..3 {
            let doc = wordcraft_docx::read(&bytes).unwrap();
            let table = doc.body[0].as_table().unwrap();
            let p = table.rows[0].props.exceptions.as_ref().unwrap();
            let expected = std::array::from_fn(|i| sides[i].unwrap_or([5.0, 4.0, 2.0, 5.4][i]));
            let st = doc.styles.table_style("Derived").unwrap();
            assert_eq!(st.parts.cell_margins, Some([1.0, 4.0, 2.0, 5.4]));
            assert_eq!(st.parts.indent, Some(0.0));
            assert_eq!(p.resolved_cell_margins(table.props.resolved_cell_margins(st.parts.cell_margins.unwrap())), expected);
            if sides.iter().any(Option::is_none) {
                assert_eq!(p.cell_margins, None);
                assert_eq!(p.partial_cell_margins, Some(sides));
            }
            bytes = wordcraft_docx::write(&doc).unwrap();
            let saved = part(&bytes, "word/document.xml");
            let margins = saved.split("<w:tblPrEx>").nth(1).unwrap().split("</w:tblPrEx>").next().unwrap();
            for (i, name) in ["top", "left", "bottom", "right"].iter().enumerate() {
                assert_eq!(margins.contains(&format!("<w:{name} ")), sides[i].is_some());
            }
        }
    }
}

#[test]
fn fidelity_script_theme_fonts_survive_styles_runs_and_repeated_saves() {
    let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let fonts = r#"<w:rFonts w:eastAsia="Fallback EA" w:cs="Fallback CS" w:eastAsiaTheme="majorEastAsia" w:cstheme="minorBidi"/>"#;
    let styles = format!(
        r#"<w:styles xmlns:w="{ns}"><w:style w:type="paragraph" w:styleId="Base"><w:name w:val="Base"/><w:rPr>{fonts}</w:rPr></w:style><w:style w:type="paragraph" w:styleId="Derived"><w:name w:val="Derived"/><w:basedOn w:val="Base"/></w:style><w:style w:type="character" w:styleId="Accent"><w:name w:val="Accent"/><w:rPr>{fonts}</w:rPr></w:style></w:styles>"#
    );
    let document = format!(
        r#"<w:document xmlns:w="{ns}"><w:body><w:p><w:pPr><w:pStyle w:val="Derived"/></w:pPr><w:r><w:rPr>{fonts}</w:rPr><w:t>Theme</w:t></w:r></w:p></w:body></w:document>"#
    );
    let mut bytes = package(&document, &styles);
    for _ in 0..3 {
        let doc = wordcraft_docx::read(&bytes).unwrap();
        for id in ["Base", "Accent"] {
            let chr = &doc.styles.get(id).unwrap().chr;
            assert_eq!(chr.font_ea_theme.as_deref(), Some("majorEastAsia"));
            assert_eq!(chr.font_cs_theme.as_deref(), Some("minorBidi"));
            assert_eq!(chr.font_ea.as_deref(), Some("Fallback EA"));
            assert_eq!(chr.font_cs.as_deref(), Some("Fallback CS"));
            let mut inherited = wordcraft_doc::props::CharProps::default();
            inherited.overlay(chr);
            assert_eq!(inherited.font_ea_theme, chr.font_ea_theme);
            inherited.overlay(&wordcraft_doc::props::CharProps { font_ea: Some("Explicit".into()), ..Default::default() });
            assert_eq!(inherited.font_ea_theme, None);
            assert_eq!(inherited.font_cs_theme, chr.font_cs_theme);
        }
        bytes = wordcraft_docx::write(&doc).unwrap();
        for name in ["word/styles.xml", "word/document.xml"] {
            let xml = part(&bytes, name);
            assert!(xml.contains("w:eastAsiaTheme=\"majorEastAsia\""));
            assert!(xml.contains("w:cstheme=\"minorBidi\""));
        }
    }
}
