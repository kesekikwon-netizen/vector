//! Trowel archaeological CAD drafting UI components.
//!
//! Implements Section 3 of `Trowel 디자인.html`:
//! 1. Left Sheet Index panel (도면집)
//! 2. Right stack panels (그리기 팔레트 · 범례·레이어 · 속성)
//! 3. Archaeological Document Tabs
//! 4. Archaeological Status Bar

use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use serde_json::json;

use crate::theme::Tokens;
use crate::{VectorcraftApp, icons};

/// Left Sheet Index panel (「도면집」 판).
pub fn sheet_index(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let frame = egui::Frame::NONE.fill(t.ground).stroke(Stroke::new(1.0, t.line));

    egui::Panel::left("trowel_sheet_index").exact_size(196.0).frame(frame).show(ui, |ui| {
        ui.vertical(|ui| {
            // Header: 도면집 광령리1 ＋
            let (hdr_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
            ui.painter().rect_filled(hdr_rect, CornerRadius::ZERO, t.ground);
            ui.painter().line_segment([hdr_rect.left_bottom(), hdr_rect.right_bottom()], Stroke::new(1.0, t.line));
            ui.painter().text(hdr_rect.left_center() + vec2(12.0, 0.0), Align2::LEFT_CENTER, "도면집", FontId::proportional(13.0), t.ink);
            ui.painter().text(hdr_rect.left_center() + vec2(54.0, 0.0), Align2::LEFT_CENTER, "광령리1", FontId::proportional(11.0), t.muted);
            let btn_rect = Rect::from_center_size(hdr_rect.right_center() - vec2(16.0, 0.0), vec2(22.0, 22.0));
            let add_resp = ui.interact(btn_rect, ui.id().with("sheet_add"), Sense::click());
            if add_resp.hovered() {
                ui.painter().rect_filled(btn_rect, CornerRadius::same(4), t.oat);
            }
            ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER, "＋", FontId::proportional(14.0), t.ink2);
            if add_resp.clicked() {
                let _ = app.run("file.newDialog", json!({}));
            }

            // Scrollable sheet list
            egui::ScrollArea::vertical().id_salt("sheet_index_scroll").auto_shrink([false, false]).max_height(ui.available_height() - 76.0).show(
                ui,
                |ui| {
                    ui.add_space(6.0);

                    // 1. 평면
                    section_header(ui, &t, "평면");
                    sheet_row(ui, &t, "유구배치도", "1:200", false, || {});
                    sheet_row(ui, &t, "1호 주거지", "1:20", app.ui.doc_tab == 0, || {
                        app.ui.doc_tab = 0;
                    });
                    sheet_row(ui, &t, "2호 주거지", "1:20", false, || {});
                    sheet_row(ui, &t, "1–3호 수혈", "1:20", false, || {});

                    ui.add_space(6.0);

                    // 2. 단면
                    section_header(ui, &t, "단면");
                    sheet_row(ui, &t, "A–A′ 1호 주거지", "1:20", app.ui.doc_tab == 1, || {
                        app.ui.doc_tab = 1;
                    });
                    sheet_row(ui, &t, "B–B′ 1호 주거지", "1:20", false, || {});
                    sheet_row(ui, &t, "Tr-1 북벽 토층", "1:20", false, || {});

                    ui.add_space(6.0);

                    // 3. 유물
                    section_header(ui, &t, "유물");
                    sheet_row(ui, &t, "1호 주거지 출토 ①–⑧", "1:1", app.ui.doc_tab == 2, || {
                        app.ui.doc_tab = 2;
                    });

                    ui.add_space(6.0);

                    // 4. 도판
                    section_header(ui, &t, "도판");
                    sheet_row(ui, &t, "도면 11 유구배치도", "", false, || {});
                    sheet_row(ui, &t, "도면 12 1호 주거지 평·단면도", "", app.ui.doc_tab == 3, || {
                        app.ui.doc_tab = 3;
                    });
                },
            );

            // Footer (lpfoot)
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                let (foot_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 74.0), Sense::hover());
                ui.painter().rect_filled(foot_rect, CornerRadius::ZERO, t.ground);
                ui.painter().line_segment([foot_rect.left_top(), foot_rect.right_top()], Stroke::new(1.0, t.line));
                let mut y = foot_rect.top() + 8.0;
                let left = foot_rect.left() + 12.0;

                let line1 = "이 도면 · 축척 1:20 · EPSG:5186 중부원점";
                ui.painter().text(egui::pos2(left, y), Align2::LEFT_TOP, line1, FontId::proportional(10.5), t.muted);
                y += 15.0;

                let line2 = "원점 X 147,990  Y 98,150";
                ui.painter().text(egui::pos2(left, y), Align2::LEFT_TOP, line2, FontId::monospace(10.5), t.ink);
                y += 15.0;

                let line3 = "높이 해발 · 지정함";
                ui.painter().text(egui::pos2(left, y), Align2::LEFT_TOP, line3, FontId::proportional(10.5), t.muted);
                y += 15.0;

                let line4 = "스캔 1장 · 2점 맞춤 · 오차 0.012 m";
                ui.painter().text(egui::pos2(left, y), Align2::LEFT_TOP, line4, FontId::proportional(10.5), t.muted);
            });
        });
    });
}

fn section_header(ui: &mut Ui, t: &Tokens, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    ui.painter().text(rect.left_center() + vec2(12.0, 0.0), Align2::LEFT_CENTER, label, FontId::proportional(11.0), t.muted);
    let line_left = rect.left() + 42.0;
    ui.painter().line_segment([egui::pos2(line_left, rect.center().y), egui::pos2(rect.right() - 12.0, rect.center().y)], Stroke::new(1.0, t.line));
}

fn sheet_row(ui: &mut Ui, t: &Tokens, title: &str, scale: &str, selected: bool, on_click: impl FnOnce()) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width() - 12.0, 26.0), Sense::click());
    let r = rect.shrink2(vec2(4.0, 0.0));
    if selected {
        ui.painter().rect_filled(r, CornerRadius::same(4), t.oat);
    } else if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(4), t.wash);
    }

    // Small icon
    let ic_box = Rect::from_center_size(r.left_center() + vec2(10.0, 0.0), vec2(13.0, 13.0));
    ui.painter().rect_stroke(ic_box, CornerRadius::same(2), Stroke::new(1.2, t.hand), StrokeKind::Inside);

    // Title
    ui.painter().text(
        r.left_center() + vec2(22.0, 0.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(11.5),
        if selected { t.ink } else { t.text },
    );

    // Scale
    if !scale.is_empty() {
        ui.painter().text(r.right_center() - vec2(6.0, 0.0), Align2::RIGHT_CENTER, scale, FontId::monospace(10.5), t.muted);
    }

    if resp.clicked() {
        on_click();
    }
}

/// Right dock column: Drawing Palette + Legend & Layers + Properties.
pub fn right_dock(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let frame = egui::Frame::NONE.fill(t.ground).stroke(Stroke::new(1.0, t.line));

    egui::Panel::right("trowel_right_dock").exact_size(310.0).frame(frame).show(ui, |ui| {
        ui.vertical(|ui| {
            // 1. Drawing Palette (그리기 팔레트)
            drawing_palette_panel(app, ui, &t);

            // 2. Legend & Layers (범례 · 레이어)
            legend_layers_panel(app, ui, &t);

            // 3. Properties (속성)
            properties_compact_panel(app, ui, &t);
        });
    });
}

fn drawing_palette_panel(app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Frame::NONE.fill(t.ground).inner_margin(egui::Margin { left: 14, right: 14, top: 4, bottom: 8 }).stroke(Stroke::new(1.0, t.line)).show(
        ui,
        |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("그리기 팔레트").size(13.0).color(t.ink).strong());
                ui.label(egui::RichText::new("다음에 그릴 선 · pt").size(11.0).color(t.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("⋯").size(14.0).color(t.ink2));
                });
            });
            ui.add_space(4.0);

            // Row 1: 굵기
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("굵기").size(11.0).color(t.muted));
                for &pt in &[0.1, 0.2, 0.3, 0.5, 0.8, 1.0] {
                    let cur_w = crate::panels::current_stroke(app).map(|s| s.width).unwrap_or(0.8);
                    let is_active = (cur_w - pt).abs() < 0.05;
                    let (cr, resp) = ui.allocate_exact_size(vec2(32.0, 22.0), Sense::click());
                    ui.painter().rect(
                        cr,
                        CornerRadius::same(4),
                        if is_active { t.oat } else { t.surface },
                        Stroke::new(1.0, if is_active { t.ring } else { t.edge }),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        cr.center(),
                        Align2::CENTER_CENTER,
                        format!("{pt:.1}"),
                        FontId::monospace(10.5),
                        if is_active { t.ink } else { t.text },
                    );
                    if resp.clicked() {
                        let _ = app.run("stroke.set", json!({ "weight": pt }));
                    }
                }
                let (plus_rect, plus_resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
                ui.painter().rect(plus_rect, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(plus_rect.center(), Align2::CENTER_CENTER, "＋", FontId::proportional(11.0), t.muted);
                if plus_resp.clicked() {
                    let _ = app.run("window.panel", json!({ "panel": "stroke" }));
                }
            });

            ui.add_space(4.0);

            // Row 2: 빗금
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("빗금").size(11.0).color(t.muted));
                for (h_label, _h_val) in [("45°", 1), ("135°", 2), ("번갈아", 3), ("없음", 0)] {
                    let is_none = h_label == "없음";
                    let (hr, _) = ui.allocate_exact_size(vec2(52.0, 22.0), Sense::click());
                    ui.painter().rect(
                        hr,
                        CornerRadius::same(4),
                        if is_none { t.oat } else { t.surface },
                        Stroke::new(1.0, if is_none { t.ring } else { t.edge }),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(hr.center(), Align2::CENTER_CENTER, h_label, FontId::proportional(11.0), t.ink);
                }
            });

            ui.add_space(4.0);

            // Row 3: 선종
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("선종").size(11.0).color(t.muted));
                let (sr, _) = ui.allocate_exact_size(vec2(210.0, 22.0), Sense::hover());
                ui.painter().rect(sr, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                let w = sr.width() / 4.0;
                for (i, name) in ["실선", "파선", "점선", "일점쇄선"].iter().enumerate() {
                    let sub_rect = Rect::from_min_size(sr.min + vec2(i as f32 * w, 0.0), vec2(w, sr.height()));
                    if i == 0 {
                        ui.painter().rect_filled(sub_rect, CornerRadius::same(4), t.oat);
                    }
                    if i > 0 {
                        ui.painter().line_segment([sub_rect.left_top(), sub_rect.left_bottom()], Stroke::new(1.0, t.line));
                    }
                    ui.painter().text(sub_rect.center(), Align2::CENTER_CENTER, *name, FontId::proportional(10.5), t.ink);
                }
            });

            ui.add_space(6.0);

            // Action Buttons: 잘린 면, 범례에 더하기, 범례대로
            ui.horizontal(|ui| {
                let (btn1, _) = ui.allocate_exact_size(vec2(68.0, 24.0), Sense::click());
                ui.painter().rect(btn1, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(btn1.center(), Align2::CENTER_CENTER, "잘린 면", FontId::proportional(11.0), t.claytext);

                let (btn2, _) = ui.allocate_exact_size(vec2(96.0, 24.0), Sense::click());
                ui.painter().rect(btn2, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(btn2.center(), Align2::CENTER_CENTER, "범례에 더하기", FontId::proportional(11.0), t.ink);

                let (btn3, _) = ui.allocate_exact_size(vec2(76.0, 24.0), Sense::click());
                ui.painter().rect(btn3, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(btn3.center(), Align2::CENTER_CENTER, "범례대로", FontId::proportional(11.0), t.ink);
            });
        },
    );
}

fn legend_layers_panel(_app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Frame::NONE.fill(t.ground).inner_margin(egui::Margin { left: 14, right: 14, top: 4, bottom: 6 }).stroke(Stroke::new(1.0, t.line)).show(
        ui,
        |ui| {
            // Header: 범례 · 레이어 평면용 ▾ ＋ ⋯
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("범례 · 레이어").size(13.0).color(t.ink).strong());
                ui.label(egui::RichText::new("평면용 ▾").size(11.0).color(t.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("⋯").size(14.0).color(t.ink2));
                    ui.label(egui::RichText::new("＋").size(14.0).color(t.ink2));
                });
            });

            // Scrollable layers list
            egui::ScrollArea::vertical().id_salt("legend_layers_scroll").auto_shrink([false, false]).max_height(280.0).show(ui, |ui| {
                section_header(ui, t, "조사 데이터 · Strata와 같은 레이어");
                legend_row(
                    ui,
                    t,
                    LegendRow { name: "조사구역 경계", weight: "1.0", count: "1", selected: false, fav: false, locked: true, expanded: false },
                );
                legend_row(ui, t, LegendRow { name: "굴광선", weight: "0.8", count: "6", selected: true, fav: true, locked: false, expanded: true });

                // Sub objects under 굴광선
                legend_sub_row(ui, t, "1호 주거지", "32.91 ㎡");
                legend_sub_row(ui, t, "2호 수혈", "0.5 따로 ↺");
                legend_sub_row(ui, t, "3호 수혈 · 그리는 중", "");

                legend_row(
                    ui,
                    t,
                    LegendRow { name: "바닥선", weight: "0.5", count: "2", selected: false, fav: true, locked: false, expanded: false },
                );
                legend_row(
                    ui,
                    t,
                    LegendRow {
                        name: "유구 선 (구·벽구)", weight: "0.5", count: "2", selected: false, fav: false, locked: false, expanded: false
                    },
                );
                legend_row(
                    ui,
                    t,
                    LegendRow { name: "단면 위치선", weight: "0.2", count: "2", selected: false, fav: true, locked: false, expanded: false },
                );
                legend_row(ui, t, LegendRow { name: "기준점", weight: "", count: "4", selected: false, fav: false, locked: true, expanded: false });
                legend_row(
                    ui,
                    t,
                    LegendRow {
                        name: "유물 출토 위치", weight: "0.2", count: "4", selected: false, fav: false, locked: false, expanded: false
                    },
                );

                ui.add_space(4.0);
                section_header(ui, t, "그리기 · Trowel");
                legend_row(
                    ui,
                    t,
                    LegendRow {
                        name: "내부 시설 (주혈·노지)",
                        weight: "0.5",
                        count: "9",
                        selected: false,
                        fav: false,
                        locked: false,
                        expanded: false,
                    },
                );
                legend_row(
                    ui,
                    t,
                    LegendRow { name: "경사 표시", weight: "0.2", count: "1", selected: false, fav: false, locked: false, expanded: false },
                );
                legend_row(ui, t, LegendRow { name: "표고점", weight: "", count: "5", selected: false, fav: false, locked: false, expanded: false });
                legend_row(
                    ui,
                    t,
                    LegendRow { name: "글자 · 번호", weight: "", count: "14", selected: false, fav: false, locked: false, expanded: false },
                );
                legend_row(
                    ui,
                    t,
                    LegendRow {
                        name: "밑그림 · 스캔 1:20", weight: "", count: "1", selected: false, fav: false, locked: true, expanded: false
                    },
                );
            });

            ui.add_space(4.0);

            // Footer: 굵기 한 번에
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("굵기 한 번에").size(10.5).color(t.muted));
                let (b1, _) = ui.allocate_exact_size(vec2(100.0, 22.0), Sense::click());
                ui.painter().rect(b1, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(b1.center(), Align2::CENTER_CENTER, "모든 굵기 ×0.8 …", FontId::proportional(10.5), t.ink);

                let (b2, _) = ui.allocate_exact_size(vec2(110.0, 22.0), Sense::click());
                ui.painter().rect(b2, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
                ui.painter().text(b2.center(), Align2::CENTER_CENTER, "따로 1 → 범례대로", FontId::proportional(10.5), t.ink);
            });
        },
    );
}

struct LegendRow<'a> {
    name: &'a str,
    weight: &'a str,
    count: &'a str,
    selected: bool,
    fav: bool,
    locked: bool,
    expanded: bool,
}

fn legend_row(ui: &mut Ui, t: &Tokens, row: LegendRow<'_>) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width() - 8.0, 22.0), Sense::click());
    if row.selected {
        ui.painter().rect_filled(rect, CornerRadius::same(4), t.oat);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(4), t.wash);
    }

    let mut x = rect.left() + 4.0;
    let arrow_txt = if row.expanded { "▾" } else { "▸" };
    ui.painter().text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, arrow_txt, FontId::proportional(9.0), t.muted);
    x += 12.0;

    // Eye icon
    let eye_box = Rect::from_center_size(egui::pos2(x + 6.0, rect.center().y), vec2(12.0, 12.0));
    icons::paint(ui, "eye", eye_box, t.ink);
    x += 16.0;

    // Lock icon
    if row.locked {
        let lock_box = Rect::from_center_size(egui::pos2(x + 5.0, rect.center().y), vec2(11.0, 11.0));
        icons::paint(ui, "lock", lock_box, t.ink);
    }
    x += 14.0;

    // Star
    if row.fav {
        ui.painter().text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, "★", FontId::proportional(10.0), t.clay);
    }
    x += 12.0;

    // Name
    let text_color = if row.fav { t.claytext } else { t.ink };
    ui.painter().text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, row.name, FontId::proportional(11.5), text_color);

    // Weight and Count on right
    let right = rect.right() - 4.0;
    ui.painter().text(egui::pos2(right, rect.center().y), Align2::RIGHT_CENTER, row.count, FontId::monospace(10.0), t.muted);
    if !row.weight.is_empty() {
        ui.painter().text(egui::pos2(right - 22.0, rect.center().y), Align2::RIGHT_CENTER, row.weight, FontId::monospace(10.0), t.muted);
    }
}

fn legend_sub_row(ui: &mut Ui, t: &Tokens, name: &str, info: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width() - 8.0, 19.0), Sense::hover());
    let x = rect.left() + 46.0;
    ui.painter().text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, name, FontId::proportional(11.0), t.muted);
    if !info.is_empty() {
        ui.painter().text(egui::pos2(rect.right() - 6.0, rect.center().y), Align2::RIGHT_CENTER, info, FontId::monospace(10.0), t.claytext);
    }
}

fn properties_compact_panel(_app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Frame::NONE.fill(t.ground).inner_margin(egui::Margin { left: 14, right: 14, top: 4, bottom: 6 }).show(ui, |ui| {
        // Header: 속성 3호 수혈 · 그리는 중 ⌄
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("속성").size(13.0).color(t.ink).strong());
            ui.label(egui::RichText::new("3호 수혈 · 그리는 중").size(11.0).color(t.muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("⌄").size(14.0).color(t.ink2));
            });
        });

        ui.add_space(4.0);

        // Key-values
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("종류").size(11.0).color(t.muted));
            let (f1, _) = ui.allocate_exact_size(vec2(100.0, 22.0), Sense::click());
            ui.painter().rect(f1, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
            ui.painter().text(f1.left_center() + vec2(8.0, 0.0), Align2::LEFT_CENTER, "수혈 ▼", FontId::proportional(11.0), t.ink);

            let (f2, _) = ui.allocate_exact_size(vec2(44.0, 22.0), Sense::click());
            ui.painter().rect(f2, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
            ui.painter().text(f2.center(), Align2::CENTER_CENTER, "3호", FontId::monospace(11.0), t.ink);
        });

        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("시대").size(11.0).color(t.muted));
            let (f3, _) = ui.allocate_exact_size(vec2(100.0, 22.0), Sense::click());
            ui.painter().rect(f3, CornerRadius::same(4), t.surface, Stroke::new(1.0, t.edge), StrokeKind::Inside);
            ui.painter().text(f3.left_center() + vec2(8.0, 0.0), Align2::LEFT_CENTER, "고르세요 ▼", FontId::proportional(11.0), t.faint);

            ui.label(egui::RichText::new("● 필요").size(10.5).color(t.claytext));
        });

        ui.add_space(4.0);
        let mini = "이 변 1.19 m · 지금까지 3.86 m · 닫으면 면적·둘레";
        ui.label(egui::RichText::new(mini).size(10.5).color(t.muted));
    });
}

/// Archaeological Document Tabs.
pub fn doc_tabs(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.wash);
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.line));

    let tabs = [("홈", ""), ("1호 주거지 평면", "1:20"), ("A–A′ 단면", "1:20"), ("도면 12", "")];

    let mut x = rect.left() + 4.0;
    for (idx, (name, sc)) in tabs.iter().enumerate() {
        let is_cur = app.ui.doc_tab == idx;
        let tab_w = 120.0 + if sc.is_empty() { 0.0 } else { 30.0 };
        let tab_r = Rect::from_min_size(egui::pos2(x, rect.top() + 2.0), vec2(tab_w, 28.0));
        let resp = ui.interact(tab_r, ui.id().with(("doc_tab", idx)), Sense::click());

        if is_cur {
            ui.painter().rect_filled(tab_r, CornerRadius::same(6), t.ground);
            ui.painter().rect_stroke(tab_r, CornerRadius::same(6), Stroke::new(1.0, t.line), StrokeKind::Inside);
        } else if resp.hovered() {
            ui.painter().rect_filled(tab_r, CornerRadius::same(6), t.oat);
        }

        let label_text = if sc.is_empty() { name.to_string() } else { format!("{name}  {sc}") };
        ui.painter().text(tab_r.center(), Align2::CENTER_CENTER, label_text, FontId::proportional(11.5), if is_cur { t.ink } else { t.muted });

        if resp.clicked() {
            app.ui.doc_tab = idx;
        }

        x += tab_w + 4.0;
    }

    // Right corner: 광령리1  ● 저장 안 됨
    let is_dirty = app.session.active().map(|d| d.is_dirty()).unwrap_or(true);
    let right_text = if is_dirty { "광령리1   ● 저장 안 됨" } else { "광령리1   저장됨" };
    let text_color = if is_dirty { t.claytext } else { t.muted };
    ui.painter().text(rect.right_center() - vec2(16.0, 0.0), Align2::RIGHT_CENTER, right_text, FontId::proportional(11.5), text_color);
}

/// Archaeological Status Bar.
pub fn status_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("trowel_status_bar").exact_size(32.0).frame(egui::Frame::NONE.fill(t.ground).stroke(Stroke::new(1.0, t.line))).show(
        ui,
        |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(12.0);

                // Left guide hint
                let hint = if !app.ui.status.is_empty() {
                    &app.ui.status
                } else {
                    "시작점을 누르면 3호 수혈이 닫히고 면적이 계산됩니다."
                };
                ui.label(egui::RichText::new(hint).size(12.0).color(t.muted));

                // Right cells
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    // Cell: 좌표계
                    status_cell(ui, &t, "5186 중부원점 · 높이 해발", t.muted);

                    // Cell: 축척
                    status_cell(ui, &t, "도면 1:20", t.ink);

                    // Cell: 좌표
                    let (x_str, y_str) = if let Some(p) = app.hover_doc {
                        // Offset for Gwangnyeong-ri 1 grid
                        let tm_x = 148_000.0 + p.x * 0.02;
                        let tm_y = 98_160.0 - p.y * 0.02;
                        (format!("{tm_x:.3}"), format!("{tm_y:.3}"))
                    } else {
                        ("148,003.214".to_string(), "98,158.407".to_string())
                    };
                    let coord_str = format!("X {}  Y {}", x_str, y_str);
                    status_cell(ui, &t, &coord_str, t.ink);

                    // Cell: 저장 상태
                    let is_dirty = app.session.active().map(|d| d.is_dirty()).unwrap_or(true);
                    let save_str = if is_dirty { "● 저장 안 됨 3건" } else { "저장됨" };
                    let save_col = if is_dirty { t.claytext } else { t.done };
                    status_cell(ui, &t, save_str, save_col);

                    // Cell: 자석
                    let snap_on = app.ui.view.snap_to_grid;
                    let snap_str = if snap_on { "자석 켬" } else { "자석 끔" };
                    status_cell(ui, &t, snap_str, t.ink);

                    // Cell: 도구
                    let tool = match app.session.tool_id() {
                        "pen" => "펜",
                        "pencil" => "연필",
                        "line" => "단면선",
                        _ => "펜",
                    };
                    status_cell(ui, &t, tool, t.ink);
                });
            });
        },
    );
}

fn status_cell(ui: &mut Ui, t: &Tokens, text: &str, color: Color32) {
    let text_galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(11.5), color);
    let cell_w = text_galley.size().x + 16.0;
    let (rect, _) = ui.allocate_exact_size(vec2(cell_w, 32.0), Sense::hover());
    ui.painter().line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.0, t.line));
    ui.painter().galley(rect.center() - vec2(text_galley.size().x / 2.0, text_galley.size().y / 2.0), text_galley, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    #[test]
    fn trowel_brightness_and_tokens() {
        assert_eq!(crate::theme::Brightness::parse("trowel"), Some(crate::theme::Brightness::Trowel));
        assert_eq!(crate::theme::Brightness::Trowel.id(), "trowel");
        assert_eq!(crate::theme::Brightness::Trowel.label(), "Trowel");

        let t = Tokens::trowel();
        assert!(!t.dark);
        assert_eq!(t.ground, Color32::from_rgb(0xFA, 0xF9, 0xF5));
        assert_eq!(t.action, Color32::from_rgb(0xB5, 0x57, 0x3A));
        assert_eq!(t.claytext, Color32::from_rgb(0x9C, 0x4A, 0x2F));
    }

    #[test]
    fn trowel_ui_components_frame_render() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.ui.brightness = crate::theme::Brightness::Trowel;
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1600.0, 988.0));

        let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| {
            crate::ribbon::show(&mut app, ui);
            sheet_index(&mut app, ui);
            right_dock(&mut app, ui);
            doc_tabs(&mut app, ui);
            status_bar(&mut app, ui);
        });
        let has_shapes = !out.shapes.is_empty();
        out.textures_delta.clear();
        assert!(has_shapes, "trowel ui elements rendered shapes");
    }
}
