//! Trowel Ribbon Toolbar: one-row grouped tile buttons for archaeological CAD drafting.

use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use serde_json::json;

use crate::theme::Tokens;
use crate::{VectorcraftApp, icons};

pub const RIBBON_HEIGHT: f32 = 72.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileStyle {
    Normal,
    Active,
    Shown,
    Save,
    Disabled,
}

pub fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let frame =
        egui::Frame::NONE.fill(t.ground).inner_margin(egui::Margin { left: 8, right: 12, top: 4, bottom: 4 }).stroke(Stroke::new(1.0, t.line));

    egui::Panel::top("trowel_ribbon").exact_size(RIBBON_HEIGHT).frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);

            // 1. 파일
            group(ui, &t, "파일", |ui| {
                if tile(ui, &t, "folder", "열기", TileStyle::Normal).clicked() {
                    let _ = app.run("file.open", json!({}));
                }
                let is_dirty = app.session.active().map(|d| d.is_dirty()).unwrap_or(false);
                let save_style = if is_dirty { TileStyle::Save } else { TileStyle::Normal };
                if tile(ui, &t, "save", "저장", save_style).clicked() {
                    let _ = app.run("file.save", json!({}));
                }
                if tile(ui, &t, "download", "가져오기", TileStyle::Normal).clicked() {
                    let _ = app.run("file.place", json!({}));
                }
            });

            separator(ui, &t);

            // 2. 새 도면
            group(ui, &t, "새 도면", |ui| {
                let cur_view = app.ui.doc_tab;
                if tile(ui, &t, "square", "평면", if cur_view == 0 { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    app.ui.doc_tab = 0;
                }
                if tile(ui, &t, "dc-line", "단면", if cur_view == 1 { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    app.ui.doc_tab = 1;
                }
                if tile(ui, &t, "circle", "유물", if cur_view == 2 { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    app.ui.doc_tab = 2;
                }
                if tile(ui, &t, "file-text", "도판", if cur_view == 3 { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    app.ui.doc_tab = 3;
                }
            });

            separator(ui, &t);

            // 3. 밑그림
            group(ui, &t, "밑그림", |ui| {
                if tile(ui, &t, "scan-line", "맞추기", TileStyle::Normal).clicked() {
                    let _ = app.run("view.actualSize", json!({}));
                }
                if tile(ui, &t, "image", "밑그림", TileStyle::Shown).clicked() {
                    let _ = app.run("view.toggleTracing", json!({}));
                }
            });

            separator(ui, &t);

            // 4. 그리기
            group(ui, &t, "그리기", |ui| {
                let tool = app.session.tool_id();
                let is_pen = tool == "pen";
                let is_pencil = tool == "pencil";
                if tile(ui, &t, "pen-tool", "펜", if is_pen { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    let _ = app.run("tool.select", json!({ "tool": "pen" }));
                }
                if tile(ui, &t, "pencil", "연필", if is_pencil { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    let _ = app.run("tool.select", json!({ "tool": "pencil" }));
                }
                if custom_tile(ui, &t, "단면선", TileStyle::Normal, |p, r, fg| {
                    // Section line dashed preview
                    p.line_segment([r.left_center() + vec2(6.0, 0.0), r.right_center() - vec2(6.0, 0.0)], Stroke::new(1.8, fg));
                    p.line_segment([r.left_center() + vec2(6.0, -6.0), r.left_center() + vec2(6.0, 6.0)], Stroke::new(2.0, fg));
                    p.line_segment([r.right_center() - vec2(6.0, -6.0), r.right_center() - vec2(6.0, 6.0)], Stroke::new(2.0, fg));
                })
                .clicked()
                {
                    let _ = app.run("tool.select", json!({ "tool": "line" }));
                }
                if custom_tile(ui, &t, "표고", TileStyle::Normal, |p, r, fg| {
                    // Elevation mark inverted triangle
                    let c = r.center();
                    p.line_segment([c + vec2(-7.0, -4.0), c + vec2(7.0, -4.0)], Stroke::new(1.6, fg));
                    p.line_segment([c + vec2(-7.0, -4.0), c + vec2(0.0, 6.0)], Stroke::new(1.6, fg));
                    p.line_segment([c + vec2(7.0, -4.0), c + vec2(0.0, 6.0)], Stroke::new(1.6, fg));
                })
                .clicked()
                {
                    let _ = app.run("tool.select", json!({ "tool": "type" }));
                }
                let is_paint = tool == "paintBucket" || tool == "livePaint";
                if tile(ui, &t, "brush", "칠통", if is_paint { TileStyle::Active } else { TileStyle::Normal }).clicked() {
                    let _ = app.run("tool.select", json!({ "tool": "livePaint" }));
                }
            });

            separator(ui, &t);

            // 5. 도면 요소
            group(ui, &t, "도면 요소", |ui| {
                if custom_tile(ui, &t, "방위", TileStyle::Normal, |p, r, fg| {
                    // North arrow symbol
                    let c = r.center();
                    p.line_segment([c + vec2(0.0, -8.0), c + vec2(-5.0, 7.0)], Stroke::new(1.5, fg));
                    p.line_segment([c + vec2(0.0, -8.0), c + vec2(5.0, 7.0)], Stroke::new(1.5, fg));
                    p.line_segment([c + vec2(0.0, -8.0), c + vec2(0.0, 5.0)], Stroke::new(1.0, fg));
                })
                .clicked()
                {
                    let _ = app.run("object.insertNorthArrow", json!({}));
                }
                if custom_tile(ui, &t, "축척", TileStyle::Normal, |p, r, fg| {
                    // Scale bar symbol
                    let c = r.center();
                    let bar = Rect::from_center_size(c, vec2(20.0, 6.0));
                    p.rect_stroke(bar, CornerRadius::ZERO, Stroke::new(1.2, fg), StrokeKind::Inside);
                    p.rect_filled(Rect::from_min_size(bar.min, vec2(10.0, 6.0)), CornerRadius::ZERO, fg);
                })
                .clicked()
                {
                    let _ = app.run("object.insertScaleBar", json!({}));
                }
                if tile(ui, &t, "table", "범례표", TileStyle::Normal).clicked() {
                    let _ = app.run("window.panel", json!({ "panel": "legend" }));
                }
                if tile(ui, &t, "layout", "표제란", TileStyle::Normal).clicked() {
                    let _ = app.run("object.insertTitleBlock", json!({}));
                }
            });

            separator(ui, &t);

            // 6. 보기
            group(ui, &t, "보기", |ui| {
                let grid_on = app.ui.view.grid;
                if tile(ui, &t, "grid", "격자", if grid_on { TileStyle::Shown } else { TileStyle::Normal }).clicked() {
                    let _ = app.run("view.showGrid", json!({}));
                }
                let snap_on = app.ui.view.snap_to_grid;
                if tile(ui, &t, "magnet", "자석", if snap_on { TileStyle::Shown } else { TileStyle::Normal }).clicked() {
                    let _ = app.run("view.snapToGrid", json!({}));
                }
            });

            separator(ui, &t);

            // 7. 내보내기
            group(ui, &t, "내보내기", |ui| {
                if custom_tile(ui, &t, "PDF", TileStyle::Normal, |p, r, fg| {
                    let c = r.center();
                    p.text(c, Align2::CENTER_CENTER, "PDF", FontId::proportional(11.0), fg);
                })
                .clicked()
                {
                    let _ = app.run("file.exportAs", json!({ "format": "pdf" }));
                }
                if custom_tile(ui, &t, "DXF", TileStyle::Normal, |p, r, fg| {
                    let c = r.center();
                    p.text(c, Align2::CENTER_CENTER, "DXF", FontId::proportional(11.0), fg);
                })
                .clicked()
                {
                    let _ = app.run("file.exportAs", json!({ "format": "dxf" }));
                }
                if tile(ui, &t, "arrow-right", "Strata로", TileStyle::Normal).clicked() {
                    let _ = app.run("file.exportAs", json!({ "format": "gpkg" }));
                }
            });

            // 8. 우측 명령 찾기
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let find_rect = ui.allocate_exact_size(vec2(140.0, 32.0), Sense::click());
                let resp = find_rect.1;
                let r = find_rect.0;
                ui.painter().rect(
                    r,
                    CornerRadius::same(8),
                    if resp.hovered() { t.oat } else { t.surface },
                    Stroke::new(1.0, t.edge),
                    StrokeKind::Inside,
                );
                ui.painter().text(r.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, "⌕ 명령 찾기", FontId::proportional(12.0), t.faint);
                if resp.clicked() {
                    app.ui.palette_open = !app.ui.palette_open;
                    app.ui.palette_query.clear();
                }
            });
        });
    });
}

fn group(ui: &mut Ui, t: &Tokens, label: &'static str, add_contents: impl FnOnce(&mut Ui)) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).font(FontId::proportional(10.5)).color(t.muted).strong());
        ui.add_space(1.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
            add_contents(ui);
        });
    });
}

fn separator(ui: &mut Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 52.0), Sense::hover());
    ui.painter().line_segment([rect.center_top() + vec2(0.0, 6.0), rect.center_bottom() - vec2(0.0, 2.0)], Stroke::new(1.0, t.line));
}

fn tile(ui: &mut Ui, t: &Tokens, icon: &str, label: &str, style: TileStyle) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(48.0, 52.0), Sense::click());
    let ic_rect = Rect::from_center_size(rect.center_top() + vec2(0.0, 18.0), vec2(36.0, 32.0));

    let (bg_fill, fg_stroke) = match style {
        TileStyle::Active => (t.hand, t.ground),
        TileStyle::Shown => (t.oat, t.ink),
        TileStyle::Save => (t.action, Color32::WHITE),
        TileStyle::Disabled => (t.wash, t.faint),
        TileStyle::Normal => {
            if resp.hovered() {
                (t.oat, t.ink)
            } else {
                (t.desk, t.hand)
            }
        }
    };

    ui.painter().rect_filled(ic_rect, CornerRadius::same(10), bg_fill);
    if style == TileStyle::Shown {
        ui.painter().rect_stroke(ic_rect, CornerRadius::same(10), Stroke::new(1.0, t.ring), StrokeKind::Inside);
    }

    if icons::exists(icon) {
        let icon_box = Rect::from_center_size(ic_rect.center(), vec2(18.0, 18.0));
        icons::paint(ui, icon, icon_box, fg_stroke);
    }

    let text_color = if style == TileStyle::Save {
        t.action
    } else if style == TileStyle::Disabled {
        t.faint
    } else {
        t.ink
    };

    let font = FontId::proportional(10.5);
    ui.painter().text(rect.center_bottom() - vec2(0.0, 2.0), Align2::CENTER_BOTTOM, label, font, text_color);

    resp
}

fn custom_tile(ui: &mut Ui, t: &Tokens, label: &str, style: TileStyle, draw_glyph: impl FnOnce(&egui::Painter, Rect, Color32)) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(48.0, 52.0), Sense::click());
    let ic_rect = Rect::from_center_size(rect.center_top() + vec2(0.0, 18.0), vec2(36.0, 32.0));

    let (bg_fill, fg_stroke) = match style {
        TileStyle::Active => (t.hand, t.ground),
        TileStyle::Shown => (t.oat, t.ink),
        TileStyle::Save => (t.action, Color32::WHITE),
        TileStyle::Disabled => (t.wash, t.faint),
        TileStyle::Normal => {
            if resp.hovered() {
                (t.oat, t.ink)
            } else {
                (t.desk, t.hand)
            }
        }
    };

    ui.painter().rect_filled(ic_rect, CornerRadius::same(10), bg_fill);
    draw_glyph(ui.painter(), ic_rect, fg_stroke);

    let text_color = if style == TileStyle::Disabled { t.faint } else { t.ink };
    let font = FontId::proportional(10.5);
    ui.painter().text(rect.center_bottom() - vec2(0.0, 2.0), Align2::CENTER_BOTTOM, label, font, text_color);

    resp
}
