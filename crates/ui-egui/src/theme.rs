//! Design tokens (five UI brightness levels: Illustrator's four plus Quiet Plate), fonts and egui style.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Brightness {
    Dark,
    MediumDark,
    MediumLight,
    Light,
    /// Quiet Plate: the default, an ivory plate with ink glyphs (not one of Illustrator's levels).
    #[default]
    Plate,
}

impl Brightness {
    pub const ALL: [Brightness; 5] = [Brightness::Dark, Brightness::MediumDark, Brightness::MediumLight, Brightness::Light, Brightness::Plate];
    pub fn label(self) -> &'static str {
        match self {
            Brightness::Dark => "Dark",
            Brightness::MediumDark => "Medium Dark",
            Brightness::MediumLight => "Medium Light",
            Brightness::Light => "Light",
            Brightness::Plate => "Quiet Plate",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Brightness::Dark => "dark",
            Brightness::MediumDark => "mediumDark",
            Brightness::MediumLight => "mediumLight",
            Brightness::Light => "light",
            Brightness::Plate => "plate",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.id().eq_ignore_ascii_case(s) || b.label().eq_ignore_ascii_case(s))
    }
}

/// All colours and metrics the UI uses. Never hard-code a colour in a widget.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub dark: bool,
    pub app_bar: Color32,
    pub panel: Color32,
    pub panel_darker: Color32,
    pub input: Color32,
    pub input_border: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub tool_active: Color32,
    /// Glyphs and text drawn over a `tool_active` well.
    pub icon_on: Color32,
    /// egui's pressed and focused widgets (`Visuals.widgets.active`); `text_strong` reads over it.
    /// The custom tool wells use `tool_active`.
    pub pressed: Color32,
    pub accent: Color32,
    pub accent_strong: Color32,
    /// egui's `Visuals.selection.bg_fill`: selected text and selectable rows.
    pub selection_fill: Color32,
    pub row_selected: Color32,
    pub pasteboard: Color32,
    pub ruler: Color32,
    pub ruler_tick: Color32,
    pub smart_guide: Color32,
    pub guide: Color32,
    pub measure_bg: Color32,
    pub button: Color32,
    pub radius: u8,
    /// egui window and menu corners.
    pub window_radius: u8,
    /// Field values, button labels, link labels.
    pub text_strong: Color32,
    /// Panel tab strips, doc-tab bar, dock header.
    pub tab_strip: Color32,
    /// Outline of secondary (outlined) buttons.
    pub button_border: Color32,
    /// 1 px edge of egui's inputs and buttons at rest; `TRANSPARENT` draws none, as Illustrator's
    /// levels do.
    pub rest_border: Color32,
    /// Default Layer 1 selection colour and selected-anchor fill.
    pub selection: Color32,
    pub anchor_selected: Color32,
    /// Active item text in tool flyouts.
    pub flyout_active: Color32,
    /// Hint-bar / tab-bar heights etc. scale with this base font size.
    pub font_size: f32,
    /// Window Close button (custom title bar) when hovered, and its glyph there.
    pub caption_close: Color32,
    pub caption_close_text: Color32,
    /// What preview images mark (Flattener Preview highlights).
    pub highlight: Color32,
    /// The bleed outline around artboards (Document Setup → Bleed).
    pub bleed: Color32,
    /// Error marks (a missing linked file).
    pub error: Color32,
    /// Warning marks (a modified linked file).
    pub warning: Color32,
    /// Rotation gizmo axes, also labelled X, Y and Z.
    pub axis_x: Color32,
    pub axis_y: Color32,
    pub axis_z: Color32,
    /// Trowel: the edge of a toggle that is on, and a legend row's eye and padlock when off.
    pub ring: Color32,
    /// Trowel: the well of the one action waiting for the user (Save with unsaved changes), and
    /// the glyph drawn on it.
    pub action: Color32,
    pub on_action: Color32,
    /// Trowel: a favourite's star and unsaved marks, and a favourite legend entry's name.
    pub clay: Color32,
    pub clay_text: Color32,
    /// Trowel (J005): the 「따로 N」 badge of a legend row with lines set apart from its look,
    /// its text, its edge and its wash.
    pub caution: Color32,
    pub caution_edge: Color32,
    pub caution_wash: Color32,
    /// Trowel (D3): the mark of a state that is as it should be (the pen as its legend row), the
    /// glyph on it `on_action`.
    pub ok: Color32,
    /// Trowel (D4): the red of the section marks on paper (the Section Line tool's icon), the same
    /// in every theme.
    pub section_mark: Color32,
    /// Trowel (D4): a tool tile's shortcut letter (the mockup's `.tl i`) and on the tile in hand
    /// (`.tl.on i`); the other themes draw them `text_disabled` and `icon_on`.
    pub tile_key: Color32,
    pub tile_key_on: Color32,
}

impl Tokens {
    pub fn for_brightness(b: Brightness) -> Self {
        let hex = |s: u32| Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8);
        let base = Tokens {
            dark: true,
            app_bar: hex(0x262626),
            panel: hex(0x323232),
            panel_darker: hex(0x282828),
            input: hex(0x1f1f1f),
            input_border: hex(0x464646),
            border: hex(0x1b1b1b),
            divider: hex(0x3f3f3f),
            text: hex(0xd8d8d8),
            text_dim: hex(0xa8a8a8),
            text_disabled: hex(0x6e6e6e),
            icon: hex(0xcfcfcf),
            hover: hex(0x444444),
            tool_active: hex(0x1d1d1d),
            icon_on: hex(0xf5f5f5),
            pressed: hex(0x1d1d1d),
            accent: hex(0x378ef0),
            accent_strong: hex(0x1473e6),
            selection_fill: hex(0x378ef0).gamma_multiply(0.55),
            row_selected: hex(0x484b50),
            pasteboard: hex(0x1e1e1e),
            ruler: hex(0x2f2f2f),
            ruler_tick: hex(0x8a8a8a),
            smart_guide: hex(0xff3dfc),
            guide: hex(0x4affff),
            measure_bg: Color32::from_rgba_unmultiplied(92, 92, 92, 235),
            button: hex(0x444444),
            radius: 2,
            window_radius: 6,
            text_strong: hex(0xf5f5f5),
            tab_strip: hex(0x282828),
            button_border: hex(0x6a6a6a),
            rest_border: Color32::TRANSPARENT,
            selection: hex(0x4f80ff),
            anchor_selected: hex(0x3d82ff),
            flyout_active: hex(0x54b2f7),
            font_size: 13.0,
            caption_close: hex(0xc42b1c),
            caption_close_text: Color32::WHITE,
            highlight: hex(0xff3030),
            bleed: hex(0xf03030),
            error: hex(0xe34850),
            warning: hex(0xf0a330),
            axis_x: hex(0xef6058),
            axis_y: hex(0x45af63),
            axis_z: hex(0x378ef0),
            ring: hex(0x6a6a6a),
            action: hex(0x1473e6),
            on_action: Color32::WHITE,
            clay: hex(0xef6058),
            clay_text: hex(0xf0a283),
            caution: hex(0xf0a330),
            caution_edge: hex(0x8a6a2a),
            caution_wash: hex(0x3f3524),
            ok: hex(0x45af63),
            section_mark: hex(0xe8334f),
            tile_key: hex(0x6e6e6e),
            tile_key_on: hex(0xf5f5f5),
        };
        let mut t = match b {
            Brightness::Dark => base,
            // Measured from Illustrator 2026 (plan/illustrator/10-observed-ui.md §1).
            Brightness::MediumDark => Tokens {
                app_bar: hex(0x535353),
                panel: hex(0x535353),
                panel_darker: hex(0x424242),
                tab_strip: hex(0x424242),
                input: hex(0x454545),
                input_border: hex(0x5f5f5f),
                border: hex(0x383838),
                divider: hex(0x4b4b4b),
                text: hex(0xd1d1d1),
                text_strong: hex(0xffffff),
                text_dim: hex(0xb0b0b0),
                text_disabled: hex(0x7a7a7a),
                icon: hex(0xc2c2c2),
                hover: hex(0x606060),
                tool_active: hex(0x303030),
                icon_on: hex(0xffffff),
                pressed: hex(0x303030),
                row_selected: hex(0x52677c),
                pasteboard: hex(0x606060),
                ruler: hex(0x333333),
                ruler_tick: hex(0x8a8a8a),
                button: hex(0x535353),
                button_border: hex(0x747474),
                ..base
            },
            Brightness::MediumLight => Tokens {
                tab_strip: hex(0xa6a6a6),
                text_strong: hex(0x000000),
                button_border: hex(0x7a7a7a),
                dark: false,
                app_bar: hex(0xa9a9a9),
                panel: hex(0xb8b8b8),
                panel_darker: hex(0xaaaaaa),
                input: hex(0xd6d6d6),
                input_border: hex(0x8c8c8c),
                border: hex(0x969696),
                divider: hex(0x9e9e9e),
                text: hex(0x1f1f1f),
                text_dim: hex(0x3c3c3c),
                text_disabled: hex(0x7c7c7c),
                icon: hex(0x2a2a2a),
                hover: hex(0xc8c8c8),
                tool_active: hex(0x969696),
                icon_on: hex(0x000000),
                pressed: hex(0x969696),
                row_selected: hex(0x9fb4d6),
                pasteboard: hex(0xa0a0a0),
                ruler: hex(0xc2c2c2),
                ruler_tick: hex(0x4a4a4a),
                button: hex(0xcacaca),
                ring: hex(0x8c8c8c),
                clay: hex(0xd97757),
                clay_text: hex(0x9c4a2f),
                caution: hex(0x7a5a00),
                caution_edge: hex(0xd9c27a),
                caution_wash: hex(0xf7efd6),
                ok: hex(0x3f6b31),
                ..base
            },
            Brightness::Light => Tokens {
                tab_strip: hex(0xdddddd),
                text_strong: hex(0x000000),
                button_border: hex(0x9a9a9a),
                dark: false,
                app_bar: hex(0xe4e4e4),
                panel: hex(0xf0f0f0),
                panel_darker: hex(0xe2e2e2),
                input: hex(0xffffff),
                input_border: hex(0xb4b4b4),
                border: hex(0xcfcfcf),
                divider: hex(0xd6d6d6),
                text: hex(0x1f1f1f),
                text_dim: hex(0x4b4b4b),
                text_disabled: hex(0x9a9a9a),
                icon: hex(0x303030),
                hover: hex(0xdddddd),
                tool_active: hex(0xcdcdcd),
                icon_on: hex(0x000000),
                pressed: hex(0xcdcdcd),
                row_selected: hex(0xc8daf7),
                pasteboard: hex(0xdcdcdc),
                ruler: hex(0xeaeaea),
                ruler_tick: hex(0x5a5a5a),
                button: hex(0xe0e0e0),
                ring: hex(0xb4b4b4),
                clay: hex(0xd97757),
                clay_text: hex(0x9c4a2f),
                caution: hex(0x7a5a00),
                caution_edge: hex(0xd9c27a),
                caution_wash: hex(0xf7efd6),
                ok: hex(0x3f6b31),
                ..base
            },
            // Quiet Plate: Trowel's own light theme (ivory ground, ink text, a clay accent on the
            // axes), every token stated here.
            Brightness::Plate => Tokens {
                dark: false,
                app_bar: hex(0xfaf9f5),
                panel: hex(0xfaf9f5),
                panel_darker: hex(0xf5f4ed),
                tab_strip: hex(0xf0eee6),
                input: hex(0xffffff),
                input_border: hex(0xc2c0b6),
                button_border: hex(0xc2c0b6),
                // Inputs and buttons show their edge at rest: white on ivory has none of its own.
                rest_border: hex(0xc2c0b6),
                border: hex(0xdedcd1),
                divider: hex(0xdedcd1),
                text: hex(0x141413),
                text_strong: hex(0x141413),
                text_dim: hex(0x5e5d59),
                text_disabled: hex(0x73726c),
                icon: hex(0x30302e),
                hover: hex(0xe8e6dc),
                tool_active: hex(0x30302e),
                // Ink glyphs vanish on the ink well: the well's glyphs are ivory.
                icon_on: hex(0xfaf9f5),
                // A pressed or focused button keeps its ink label: the press face, not the hand well.
                pressed: hex(0xe8e6dc),
                accent: hex(0x30302e),
                accent_strong: hex(0x141413),
                // A translucent ink accent under ink text can't be read: an oat fill instead.
                selection_fill: hex(0xe3dacc),
                row_selected: hex(0xe3dacc),
                pasteboard: hex(0xf0eee6),
                ruler: hex(0xf5f4ed),
                ruler_tick: hex(0x87867f),
                guide: hex(0x6a9bcc),
                smart_guide: hex(0xc46686),
                selection: hex(0xb5573a),
                anchor_selected: hex(0xb5573a),
                flyout_active: hex(0x9c4a2f),
                measure_bg: Color32::from_rgba_unmultiplied(0x14, 0x14, 0x13, 235),
                button: hex(0xf0eee6),
                radius: 8,
                window_radius: 8,
                font_size: 13.0,
                caption_close: hex(0xa33b3b),
                caption_close_text: Color32::WHITE,
                highlight: hex(0xa33b3b),
                bleed: hex(0xa33b3b),
                error: hex(0xa33b3b),
                warning: hex(0x7a5a00),
                axis_x: hex(0xd97757),
                axis_y: hex(0x3f6b31),
                axis_z: hex(0x6a9bcc),
                ring: hex(0xb0aea5),
                action: hex(0xb5573a),
                on_action: Color32::WHITE,
                clay: hex(0xd97757),
                clay_text: hex(0x9c4a2f),
                caution: hex(0x7a5a00),
                caution_edge: hex(0xd9c27a),
                caution_wash: hex(0xf7efd6),
                ok: hex(0x3f6b31),
                section_mark: hex(0xe8334f),
                tile_key: hex(0x9a988f),
                tile_key_on: hex(0xcfcfc8),
            },
        };
        if b != Brightness::Plate {
            (t.tile_key, t.tile_key_on) = (t.text_disabled, t.icon_on);
        }
        t
    }

    pub fn get(ctx: &egui::Context) -> Tokens {
        ctx.data(|d| d.get_temp::<Tokens>(egui::Id::NULL)).unwrap_or_else(|| Tokens::for_brightness(Brightness::default()))
    }
    pub fn cr(&self) -> CornerRadius {
        CornerRadius::same(self.radius)
    }
    /// Corners of small marks (check boxes): the control radius capped at 4, so a 14 px box keeps
    /// its corners.
    pub fn mark_cr(&self) -> CornerRadius {
        CornerRadius::same(self.radius.min(4))
    }
}

pub const FONT_UI: &str = "ui";
pub const FONT_UI_SEMIBOLD: &str = "ui-semibold";
pub const FONT_MONO: &str = "mono";

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_UI_SEMIBOLD.into()))
}
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_MONO.into()))
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let add = |f: &mut FontDefinitions, name: &str, bytes: &'static [u8]| {
        f.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    add(&mut fonts, "SourceSans3", include_bytes!("../../../assets/fonts/SourceSans3-Regular.ttf"));
    add(&mut fonts, "SourceSans3-Semibold", include_bytes!("../../../assets/fonts/SourceSans3-Semibold.ttf"));
    add(&mut fonts, "JetBrainsMono", include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf"));
    let fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut prop = vec!["SourceSans3".to_string()];
    prop.extend(fallback.clone());
    fonts.families.insert(FontFamily::Proportional, prop);
    let mut semi = vec!["SourceSans3-Semibold".to_string()];
    semi.extend(fallback.clone());
    fonts.families.insert(FontFamily::Name(FONT_UI_SEMIBOLD.into()), semi);
    let mut ui = vec!["SourceSans3".to_string()];
    ui.extend(fallback.clone());
    fonts.families.insert(FontFamily::Name(FONT_UI.into()), ui);
    let mut mono = vec!["JetBrainsMono".to_string()];
    mono.extend(fallback);
    fonts.families.insert(FontFamily::Name(FONT_MONO.into()), mono);
    add_craft_fonts(&mut fonts);
    ctx.set_fonts(fonts);
}

/// The egui name of a craft-fonts face.
pub(crate) fn craft_font_name(f: &vectorcraft_text::CraftFont) -> String {
    format!("craft-fonts {} {}", f.family, f.style)
}

/// The Japanese craft-fonts faces (when built with `CRAFT_FONTS_DIR`) as fallbacks at the end of
/// every family, after the app's own fonts and before the installed fonts `ui_fonts` adds: BIZ
/// UDPGothic first (bold first in the semibold family), then the Mincho faces.
fn add_craft_fonts(fonts: &mut FontDefinitions) {
    for f in vectorcraft_text::CRAFT_FONTS.iter().filter(|f| f.is_japanese()) {
        fonts.font_data.insert(craft_font_name(f), Arc::new(FontData::from_static(f.bytes)));
    }
    for (family, stack) in fonts.families.iter_mut() {
        let bold = *family == FontFamily::Name(FONT_UI_SEMIBOLD.into());
        stack.extend(vectorcraft_text::craft_fonts::japanese_ui_fonts(bold).into_iter().map(craft_font_name));
    }
}

/// Apply tokens to egui's global style.
pub fn apply(ctx: &egui::Context, b: Brightness) {
    let t = Tokens::for_brightness(b);
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, t));
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.panel;
    v.window_fill = t.panel;
    v.extreme_bg_color = t.input;
    v.faint_bg_color = t.panel_darker;
    v.window_stroke = Stroke::new(1.0, t.border);
    v.window_corner_radius = CornerRadius::same(t.window_radius);
    v.menu_corner_radius = CornerRadius::same(t.window_radius);
    v.selection.bg_fill = t.selection_fill;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.hyperlink_color = t.accent;
    v.override_text_color = Some(t.text);
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(if t.dark { 120 } else { 60 }) };
    v.window_shadow = egui::epaint::Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(if t.dark { 140 } else { 70 }) };
    let w = &mut v.widgets;
    for (wv, fill) in
        [(&mut w.noninteractive, t.panel), (&mut w.inactive, t.button), (&mut w.hovered, t.hover), (&mut w.active, t.pressed), (&mut w.open, t.hover)]
    {
        wv.bg_fill = fill;
        wv.weak_bg_fill = fill;
        wv.corner_radius = t.cr();
        wv.fg_stroke = Stroke::new(1.0, t.text);
    }
    // egui draws its pressed and focused glyphs (check marks, radio dots, resize handles) with this
    // stroke, and `.strong()` text takes its colour as `Visuals::strong_text_color`: it has to read
    // over `pressed` and over the panel, which `text_strong` does. (The custom tool wells draw
    // `icon_on` over `tool_active` themselves.)
    w.active.fg_stroke = Stroke::new(1.0, t.text_strong);
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.divider);
    w.inactive.bg_stroke = if t.rest_border == Color32::TRANSPARENT { Stroke::NONE } else { Stroke::new(1.0, t.rest_border) };
    w.hovered.bg_stroke = Stroke::new(1.0, t.input_border);
    w.active.bg_stroke = Stroke::new(1.0, t.accent);
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(6.0, 2.0);
        s.spacing.interact_size = egui::vec2(24.0, 24.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.slider_width = 120.0;
        s.text_styles = [
            (TextStyle::Small, FontId::proportional(11.0)),
            (TextStyle::Body, FontId::proportional(13.0)),
            (TextStyle::Button, FontId::proportional(13.0)),
            (TextStyle::Heading, FontId::new(15.0, FontFamily::Name(FONT_UI_SEMIBOLD.into()))),
            (TextStyle::Monospace, FontId::new(11.5, FontFamily::Name(FONT_MONO.into()))),
        ]
        .into();
        s.animation_time = 0.08;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: u32) -> Color32 {
        Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8)
    }

    #[test]
    fn brightness_roundtrip() {
        for b in Brightness::ALL {
            assert_eq!(Brightness::parse(b.id()), Some(b));
            assert_eq!(Brightness::parse(b.label()), Some(b));
        }
        assert!(!Tokens::for_brightness(Brightness::Light).dark);
        // Quiet Plate: the fifth level and the default of a fresh install.
        assert_eq!(Brightness::ALL.len(), 5);
        assert_eq!(Brightness::default(), Brightness::Plate);
        assert_eq!(Brightness::parse("plate"), Some(Brightness::Plate));
        assert_eq!(Brightness::parse("Quiet Plate"), Some(Brightness::Plate));
        assert_eq!(Brightness::Plate.label(), "Quiet Plate");
        assert_eq!(Brightness::Plate.id(), "plate");
        let plate = Tokens::for_brightness(Brightness::Plate);
        assert!(!plate.dark);
        assert_ne!(plate.icon_on, plate.tool_active, "glyphs show on the pressed well");
        assert_eq!(plate.selection_fill, hex(0xE3DACC));
        assert_eq!(plate.radius, 8);
        assert_eq!(plate.window_radius, 8);
        // Review 1: a pressed egui button is the press face under its ink label, not the hand
        // well; inputs and buttons show their edge at rest; a 14 px check box keeps its corners.
        assert_eq!(plate.pressed, hex(0xE8E6DC));
        assert_eq!(plate.rest_border, plate.input_border);
        assert_eq!(plate.mark_cr(), CornerRadius::same(4));
    }

    /// Design 2.1's marks beyond the shared plate: the edge of a toggle that is on, the save
    /// tile's well and glyph, the clay star and the clay text of a favourite legend row.
    #[test]
    fn plate_has_the_trowel_tokens() {
        let plate = Tokens::for_brightness(Brightness::Plate);
        assert_eq!(plate.ring, hex(0xB0AEA5));
        assert_eq!(plate.action, hex(0xB5573A));
        assert_eq!(plate.on_action, hex(0xFFFFFF));
        assert_eq!(plate.clay, hex(0xD97757));
        assert_eq!(plate.clay_text, hex(0x9C4A2F));
        for b in Brightness::ALL {
            let t = Tokens::for_brightness(b);
            assert!(contrast(t.on_action, t.action) >= 3.0, "{b:?}: the save glyph on its well");
            assert!(contrast(t.clay_text, t.panel) >= 3.0, "{b:?}: a favourite's name on the panel");
        }
    }

    /// J005: the 「따로 N」 badge on a legend row (lines set apart from their row's look) is a
    /// caution mark, its text and edge on a wash: Plate's are design 2.1's, and every level's text
    /// reads on its wash and the wash shows on the panel.
    #[test]
    fn every_preset_has_the_caution_tokens() {
        let plate = Tokens::for_brightness(Brightness::Plate);
        assert_eq!(plate.caution, hex(0x7A5A00));
        assert_eq!(plate.caution_edge, hex(0xD9C27A));
        assert_eq!(plate.caution_wash, hex(0xF7EFD6));
        for b in Brightness::ALL {
            let t = Tokens::for_brightness(b);
            assert!(contrast(t.caution, t.caution_wash) >= 4.5, "{b:?}: the badge text on its wash");
            assert!(contrast(t.caution, t.panel) >= 3.0, "{b:?}: the badge text on the panel");
            assert_ne!(t.caution_wash, t.panel, "{b:?}: the badge shows on the panel");
            assert_ne!(t.caution_edge, t.caution_wash, "{b:?}: the badge has an edge");
        }
    }

    /// The four Illustrator levels keep the values measured from public screenshots: the new
    /// tokens only name what `apply` already drew for them.
    #[test]
    fn old_presets_keep_their_rendering() {
        for b in [Brightness::Dark, Brightness::MediumDark, Brightness::MediumLight, Brightness::Light] {
            let t = Tokens::for_brightness(b);
            assert_eq!(t.icon_on, t.text_strong, "{b:?}");
            assert_eq!(t.selection_fill, t.accent.gamma_multiply(0.55), "{b:?}");
            assert_eq!(t.window_radius, 6, "{b:?}");
            assert_eq!(t.radius, 2, "{b:?}");
            assert_eq!(t.accent, hex(0x378ef0), "{b:?}");
            assert_eq!(t.pressed, t.tool_active, "{b:?}");
            assert_eq!(t.rest_border, Color32::TRANSPARENT, "{b:?}");
            assert_eq!(t.mark_cr(), t.cr(), "{b:?}");
        }
    }

    /// WCAG 2.x contrast ratio of `fg` over `bg` (both opaque).
    fn contrast(fg: Color32, bg: Color32) -> f32 {
        fn luminance(c: Color32) -> f32 {
            let ch = |v: u8| {
                let v = v as f32 / 255.0;
                if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
        }
        let (a, b) = (luminance(fg) + 0.05, luminance(bg) + 0.05);
        a.max(b) / a.min(b)
    }

    #[test]
    fn apply_takes_window_radius_selection_fill_pressed_and_rest_border_from_tokens() {
        let ctx = egui::Context::default();
        apply(&ctx, Brightness::Plate);
        let t = Tokens::for_brightness(Brightness::Plate);
        let v = ctx.global_style().visuals.clone();
        assert_eq!(v.window_corner_radius, CornerRadius::same(8));
        assert_eq!(v.menu_corner_radius, CornerRadius::same(8));
        assert_eq!(v.selection.bg_fill, t.selection_fill);
        // A pressed or focused egui button: the press face, its label (`override_text_color`)
        // still readable (WCAG AA), its edge the accent.
        assert_eq!(v.widgets.active.bg_fill, t.pressed);
        let label = v.override_text_color.expect("the UI overrides egui's text colour");
        assert!(contrast(label, v.widgets.active.bg_fill) >= 4.5, "{label:?} on {:?}", v.widgets.active.bg_fill);
        assert_eq!(v.widgets.active.bg_stroke.color, t.accent);
        // Inputs and buttons at rest carry their edge; hovered keeps it, focused takes the accent.
        assert_eq!(v.widgets.inactive.bg_stroke, Stroke::new(1.0, t.input_border));
        assert_eq!(v.widgets.hovered.bg_stroke.color, t.input_border);
        apply(&ctx, Brightness::Light);
        let light = Tokens::for_brightness(Brightness::Light);
        let v = ctx.global_style().visuals.clone();
        assert_eq!(v.window_corner_radius, CornerRadius::same(6));
        assert_eq!(v.widgets.active.bg_fill, light.tool_active, "Illustrator's levels press the well");
        assert_eq!(v.widgets.inactive.bg_stroke, Stroke::NONE, "…and draw no edge at rest");
    }

    /// `widgets.active.fg_stroke` is also egui's `strong_text_color`, which `.strong()` text takes
    /// before `override_text_color`: it is `text_strong` in every level, so a `.strong()` label
    /// without a `.color(..)` of its own stays readable on the panel, and so do egui's own check
    /// marks and radio dots over `pressed`.
    #[test]
    fn strong_text_and_pressed_glyphs_are_text_strong_in_every_level() {
        let ctx = egui::Context::default();
        for b in Brightness::ALL {
            apply(&ctx, b);
            let t = Tokens::for_brightness(b);
            let v = ctx.global_style().visuals.clone();
            assert_eq!(v.strong_text_color(), t.text_strong, "{b:?}");
            assert_eq!(v.widgets.active.fg_stroke.color, t.text_strong, "{b:?}");
            assert!(contrast(t.text_strong, t.panel) >= 4.5, "{b:?}: strong text on the panel");
            assert!(contrast(t.text_strong, t.pressed) >= 3.0, "{b:?}: a check mark over the pressed face");
        }
    }
}
