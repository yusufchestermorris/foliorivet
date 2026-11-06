//! Narrow OOXML color provenance. Resolved colors remain the rendering input.
use super::{Deserialize, Rgb, Serialize, TextColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeSlot {
    Dark1,
    Light1,
    Dark2,
    Light2,
    Text1,
    Background1,
    Text2,
    Background2,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
    Accent6,
    Hyperlink,
    FollowedHyperlink,
}
impl ThemeSlot {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dark1 => "dark1",
            Self::Light1 => "light1",
            Self::Dark2 => "dark2",
            Self::Light2 => "light2",
            Self::Text1 => "text1",
            Self::Background1 => "background1",
            Self::Text2 => "text2",
            Self::Background2 => "background2",
            Self::Accent1 => "accent1",
            Self::Accent2 => "accent2",
            Self::Accent3 => "accent3",
            Self::Accent4 => "accent4",
            Self::Accent5 => "accent5",
            Self::Accent6 => "accent6",
            Self::Hyperlink => "hyperlink",
            Self::FollowedHyperlink => "followedHyperlink",
        }
    }
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::Dark1,
            Self::Light1,
            Self::Dark2,
            Self::Light2,
            Self::Text1,
            Self::Background1,
            Self::Text2,
            Self::Background2,
            Self::Accent1,
            Self::Accent2,
            Self::Accent3,
            Self::Accent4,
            Self::Accent5,
            Self::Accent6,
            Self::Hyperlink,
            Self::FollowedHyperlink,
        ]
        .into_iter()
        .find(|s| s.name() == name)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OoxmlColor {
    pub slot: ThemeSlot,
    pub tint: Option<u8>,
    pub shade: Option<u8>,
    pub fallback: Option<TextColor>,
    /// Snapshot for invalidation when the resolved color is edited.
    pub resolved: Option<TextColor>,
}
impl OoxmlColor {
    pub fn matches_rgb(self, color: Option<Rgb>) -> bool {
        self.resolved == color.map(TextColor::Rgb)
    }
}
