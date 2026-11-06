use crate::Session;
use serde_json::json;
use wordcraft_doc::props::{CharProps, OoxmlColor, OoxmlFonts, Rgb, TextColor, ThemeSlot};
use wordcraft_doc::{Document, Pos};

#[test]
fn style_fidelity_same_value_font_color_edits_and_undo_replace_provenance() {
    let font = "Fixture Font".to_string();
    let color = TextColor::Rgb(Rgb(0x30, 0x60, 0x90));
    let props = CharProps {
        font: Some(font.clone()),
        color: Some(color),
        font_source: Some(Box::new(OoxmlFonts { ascii_theme: Some("minorHAnsi".into()), resolved: Some(font.clone()), ..Default::default() })),
        color_source: Some(OoxmlColor { slot: ThemeSlot::Accent1, tint: None, shade: None, fallback: None, resolved: Some(color) }),
        ..Default::default()
    };
    let mut doc = Document::new();
    doc.insert_text(&Pos::body(0, 0), "Text", &props).unwrap();
    let mut s = Session::new(doc);
    s.run("select.all", &json!({})).unwrap();
    s.run("format.font", &json!({"name": font})).unwrap();
    assert!(s.doc.para_at(&Pos::body(0, 0)).unwrap().props_of_char(0).font_source.is_none());
    s.run("edit.undo", &json!({})).unwrap();
    assert!(s.doc.para_at(&Pos::body(0, 0)).unwrap().props_of_char(0).font_source.is_some());
    s.run("format.color", &json!({"color":"306090"})).unwrap();
    assert!(s.doc.para_at(&Pos::body(0, 0)).unwrap().props_of_char(0).color_source.is_none());
    s.run("edit.undo", &json!({})).unwrap();
    assert!(s.doc.para_at(&Pos::body(0, 0)).unwrap().props_of_char(0).color_source.is_some());
}

#[test]
fn style_fidelity_style_numbering_edits_clear_level_only_source() {
    use wordcraft_doc::styles::{Style, StyleOoxml};
    let mut doc = Document::new();
    std::sync::Arc::make_mut(&mut doc.styles).styles.push(Style {
        id: "Preserved".into(),
        name: "Preserved".into(),
        ooxml: Some(StyleOoxml { numbering_level: Some(1), ..Default::default() }),
        ..Default::default()
    });
    let mut s = Session::new(doc);
    s.run("styles.modify", &json!({"style":"Preserved", "para":{"numbering":{"num":0,"level":0}}})).unwrap();
    assert!(s.doc.styles.get("Preserved").unwrap().ooxml.as_ref().unwrap().numbering_level.is_none());
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc.styles.get("Preserved").unwrap().ooxml.as_ref().unwrap().numbering_level, Some(1));
}
