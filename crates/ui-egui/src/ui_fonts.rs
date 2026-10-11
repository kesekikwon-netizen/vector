//! Fallback fonts for the app's own text: layer and file names (and anything else the UI shows) in
//! Chinese, Japanese, Korean or other scripts the UI fonts lack are drawn in an installed font that
//! covers them. Builds made with craft-fonts (`CRAFT_FONTS_DIR`, all releases) carry Japanese fonts,
//! which `theme::install_fonts` adds after the UI fonts, so Japanese never gets here; nothing else
//! is bundled for them: the first time a frame paints a character the UI fonts
//! don't have, an installed font covering it is looked for (in the background on native) and added
//! to egui's families as their last fallback, from the next frame on. Text the UI fonts cover costs
//! no font loading or memory. On the web, which has no system fonts, such characters stay
//! missing-glyph boxes (Japanese too, unless built with craft-fonts).

use std::collections::HashSet;
use std::sync::Arc;

use egui::epaint::Shape;
use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{FontData, FontFamily, FontId, LayerId};
use vectorcraft_text::{FontDb, FontFace};

/// A fallback face and its family's bold (for bold UI text), if the family has one.
type Fallback = (Arc<FontFace>, Option<Arc<FontFace>>);

#[derive(Default)]
pub(crate) struct UiFonts {
    /// Characters looked at already: the UI fonts have them, a fallback was added (or is being
    /// looked for), or no font has them.
    seen: HashSet<char>,
    /// The fonts added to egui, by name.
    added: HashSet<String>,
    /// The faces covering the characters being looked for, each with its bold (native: looked for
    /// on a thread).
    #[cfg(not(target_arch = "wasm32"))]
    pending: Option<std::sync::mpsc::Receiver<Vec<Fallback>>>,
}

impl UiFonts {
    /// After a frame is laid out: add the fallback fonts found since the last frame, and look for
    /// fonts covering the characters this frame painted that the UI fonts lack.
    pub(crate) fn frame(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(faces) = self.pending.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.pending = None;
            self.install(ctx, &faces);
        }
        let missing = self.missing(ctx);
        if missing.is_empty() {
            return;
        }
        let find = move || {
            let db = FontDb::global();
            let mut faces: Vec<Fallback> = vec![];
            for c in missing {
                if let Some(f) = db.face_covering(c)
                    && !faces.iter().any(|(x, _)| x.id() == f.id())
                {
                    let bold = bold_face(db, &f);
                    faces.push((f, bold));
                }
            }
            faces
        };
        // Native: the installed fonts are read off the UI thread. Characters met meanwhile wait for
        // the next search (they aren't marked seen until then).
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            let repaint = ctx.clone();
            let spawned = std::thread::Builder::new().name("ui-font-fallback".into()).spawn(move || {
                // The receiver is gone only when the app is.
                let _ = tx.send(find());
                repaint.request_repaint();
            });
            if spawned.is_ok() {
                self.pending = Some(rx);
            }
        }
        // The web has only the loaded fonts to look through: quick, on this thread.
        #[cfg(target_arch = "wasm32")]
        self.install(ctx, &find());
    }

    /// Characters painted this frame that the UI fonts lack and haven't been looked for, now
    /// marked seen. None while a search is running.
    fn missing(&mut self, ctx: &egui::Context) -> Vec<char> {
        #[cfg(not(target_arch = "wasm32"))]
        if self.pending.is_some() {
            return vec![];
        }
        let mut layers: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
        if !layers.contains(&LayerId::background()) {
            layers.push(LayerId::background());
        }
        let mut chars: Vec<char> = vec![];
        let seen = &self.seen;
        let mut note = |text: &str| {
            // Most UI text is ASCII, which the UI fonts cover.
            if !text.is_ascii() {
                chars.extend(text.chars().filter(|c| !c.is_ascii() && !c.is_whitespace() && !c.is_control() && !seen.contains(c)));
            }
        };
        ctx.graphics(|g| {
            for l in &layers {
                for s in g.get(*l).into_iter().flat_map(|list| list.all_entries()) {
                    painted_text(&s.shape, &mut note);
                }
            }
        });
        if chars.is_empty() {
            return chars;
        }
        chars.sort_unstable();
        chars.dedup();
        let font = FontId::new(13.0, FontFamily::Proportional);
        let missing: Vec<char> = ctx.fonts_mut(|f| chars.iter().copied().filter(|c| !f.has_glyph(&font, *c)).collect());
        self.seen.extend(chars);
        missing
    }

    /// Add `faces` to every egui family, after the fonts it has; a face's bold goes before it in the
    /// semibold family (bold UI text).
    fn install(&mut self, ctx: &egui::Context, faces: &[Fallback]) {
        let all: Vec<FontFamily> = ctx.fonts(|f| f.definitions().families.keys().cloned().collect());
        let semibold = [FontFamily::Name(crate::theme::FONT_UI_SEMIBOLD.into())];
        for (face, bold) in faces {
            if let Some(bold) = bold {
                self.add(ctx, bold, &semibold);
            }
            self.add(ctx, face, &all);
        }
        if !faces.is_empty() {
            ctx.request_repaint();
        }
    }

    /// Add `face` to `families`, after the fonts they have (once).
    fn add(&mut self, ctx: &egui::Context, face: &Arc<FontFace>, families: &[FontFamily]) {
        let name = format!("{} {} #{}", face.family, face.style, face.face_index());
        if !self.added.insert(name.clone()) {
            return;
        }
        // The face's file data, shared with the process-wide font database (which keeps every face
        // it loads for the session) instead of copied: one handle is leaked per font added.
        let face: &'static FontFace = Box::leak(Box::new(face.clone()));
        let data = FontData { index: face.face_index(), ..FontData::from_static(face.file_data()) };
        let families = families.iter().map(|family| InsertFontFamily { family: family.clone(), priority: FontPriority::Lowest }).collect();
        ctx.add_font(FontInsert { name, data, families });
    }

    /// Wait for the font search running, if any, and add what it found (tests).
    #[cfg(test)]
    pub(crate) fn finish(&mut self, ctx: &egui::Context) {
        if let Some(faces) = self.pending.take().and_then(|rx| rx.recv().ok()) {
            self.install(ctx, &faces);
        }
    }
}

/// The bold of a fallback `face`: its family's Bold style (SemiBold and up), if it has one.
fn bold_face(db: &FontDb, face: &FontFace) -> Option<Arc<FontFace>> {
    db.face(&face.family, "Bold").filter(|b| b.family.eq_ignore_ascii_case(&face.family) && !b.italic && b.weight >= 600.0 && b.id() != face.id())
}

/// Call `f` with the text of `shape` and the shapes it holds.
fn painted_text(shape: &Shape, f: &mut impl FnMut(&str)) {
    match shape {
        Shape::Text(t) => f(t.galley.text()),
        Shape::Vec(v) => v.iter().for_each(|s| painted_text(s, f)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a frame painting `text`, then the fallback search after it.
    fn frame(ctx: &egui::Context, fonts: &mut UiFonts, text: &str) {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label(text);
            fonts.frame(ui.ctx());
        });
        out.textures_delta.clear();
    }

    fn has_glyph(ctx: &egui::Context, c: char) -> bool {
        ctx.fonts_mut(|f| f.has_glyph(&FontId::new(13.0, FontFamily::Proportional), c))
    }

    #[test]
    fn latin_text_loads_no_fonts() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut fonts = UiFonts::default();
        frame(&ctx, &mut fonts, "Layer 1 — Café");
        assert!(fonts.pending.is_none() && fonts.added.is_empty());
    }

    #[test]
    fn japanese_renders_with_the_craft_fonts_without_installed_fonts() {
        if !vectorcraft_text::CRAFT_FONTS.iter().any(|f| f.is_japanese()) {
            eprintln!("skipped: built without craft-fonts (set CRAFT_FONTS_DIR to a craft-fonts checkout to run it)");
            return;
        }
        let text = "日本語の文字";
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut fonts = UiFonts::default();
        frame(&ctx, &mut fonts, text);
        // Real glyphs (no tofu) in every family from the first frame, with no installed font looked for.
        assert!(fonts.pending.is_none() && fonts.added.is_empty(), "no system-font search");
        let families: Vec<FontFamily> = ctx.fonts(|f| f.definitions().families.keys().cloned().collect());
        for family in families {
            let stack = ctx.fonts(|f| f.definitions().families.get(&family).cloned().unwrap_or_default());
            assert!(stack.last().is_some_and(|n| n.starts_with("craft-fonts ")), "{family:?}: craft-fonts last: {stack:?}");
            for c in text.chars() {
                assert!(ctx.fonts_mut(|f| f.has_glyph(&FontId::new(13.0, family.clone()), c)), "{family:?} draws {c}");
            }
        }
        // The UI prefers BIZ UDPGothic.
        let stack = ctx.fonts(|f| f.definitions().families.get(&FontFamily::Proportional).cloned().unwrap_or_default());
        let first = stack.iter().find(|n| n.starts_with("craft-fonts ")).cloned().unwrap_or_default();
        assert_eq!(first, "craft-fonts BIZ UDPGothic Regular", "{stack:?}");
    }

    #[test]
    fn the_ui_works_without_the_craft_fonts() {
        // Built either way, the fonts install and Japanese text lays out (as installed-font
        // fallbacks or missing-glyph boxes when no font has it).
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut fonts = UiFonts::default();
        frame(&ctx, &mut fonts, "日本語の文字 Layer 1");
        fonts.finish(&ctx);
        frame(&ctx, &mut fonts, "日本語の文字 Layer 1");
        if vectorcraft_text::CRAFT_FONTS.is_empty() {
            let names: Vec<String> = ctx.fonts(|f| f.definitions().font_data.keys().cloned().collect());
            assert!(names.iter().all(|n| !n.starts_with("craft-fonts")), "{names:?}");
        }
        assert!(has_glyph(&ctx, 'L'));
    }

    #[test]
    fn cjk_names_get_an_installed_fallback_font() {
        let text = "Logo 标志 ロゴ 로고";
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut fonts = UiFonts::default();
        frame(&ctx, &mut fonts, text);
        assert!(!has_glyph(&ctx, '标'), "the UI fonts have no CJK");
        fonts.finish(&ctx);
        // The fonts arrive with the next frame.
        frame(&ctx, &mut fonts, text);
        let installed: Vec<char> = text.chars().filter(|c| !c.is_ascii() && FontDb::global().face_covering(*c).is_some()).collect();
        if installed.is_empty() {
            eprintln!("no installed font covers {text}: nothing to check");
            return;
        }
        for c in installed {
            assert!(has_glyph(&ctx, c), "{c} has a glyph");
        }
        // Looked for once: a later frame starts no search.
        frame(&ctx, &mut fonts, text);
        assert!(fonts.pending.is_none());
    }

    /// A fallback face's bold is its family's Bold style; a family without a bold style has none
    /// (the bold UI text then uses the regular face).
    #[test]
    fn a_fallback_faces_bold_is_its_familys_bold_style() {
        let db = FontDb::with_font_dirs(vec![]);
        let face = |family: &str| db.face(family, "Regular").filter(|f| f.family == family).unwrap();
        let bold = bold_face(&db, &face("Source Sans 3")).unwrap();
        assert_eq!((bold.family.as_str(), bold.style.as_str()), ("Source Sans 3", "Bold"));
        // JetBrains Mono comes in Regular only.
        assert!(bold_face(&db, &face("JetBrains Mono")).is_none());
    }

    /// Bold UI text (the semibold family: titles, the current layer) in a script the UI fonts lack
    /// is drawn in the bold face of the installed family covering it; the other families keep its
    /// regular face.
    #[test]
    fn the_semibold_family_draws_with_the_fallback_familys_bold_face() {
        let db = FontDb::with_font_dirs(vec![]);
        let regular = db.face("Source Sans 3", "Regular").unwrap();
        let bold = db.face("Source Sans 3", "Bold").unwrap();
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut fonts = UiFonts::default();
        frame(&ctx, &mut fonts, "Layer 1");
        fonts.install(&ctx, &[(regular.clone(), Some(bold.clone()))]);
        // The fonts arrive with the next frame.
        frame(&ctx, &mut fonts, "Layer 1");
        let stack = |family: FontFamily| ctx.fonts(|f| f.definitions().families.get(&family).cloned().unwrap_or_default());
        let name = |f: &FontFace| format!("{} {} #{}", f.family, f.style, f.face_index());
        let (regular, bold) = (name(&regular), name(&bold));
        let semibold = stack(FontFamily::Name(crate::theme::FONT_UI_SEMIBOLD.into()));
        let at = |n: &str| semibold.iter().position(|x| x == n);
        assert!(at(&bold).is_some_and(|b| at(&regular).is_none_or(|r| b < r)), "bold before regular: {semibold:?}");
        for family in [FontFamily::Proportional, FontFamily::Name(crate::theme::FONT_UI.into()), FontFamily::Name(crate::theme::FONT_MONO.into())] {
            let s = stack(family.clone());
            assert!(s.contains(&regular) && !s.contains(&bold), "{family:?}: {s:?}");
        }
    }
}
