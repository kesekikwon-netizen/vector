//! The drawing set: the open drawings, the current one marked; a click shows one (from the Home
//! screen too) and ＋ starts a new one. Its folders, templates and the scale and coordinate system
//! at the bottom come with J004 and J006.

use serde_json::json;

use super::*;

const HEAD_H: f32 = 32.0;
const ROW_H: f32 = 26.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::left(SET)
        .resizable(true)
        .default_size(SET_W)
        .size_range(140.0..=450.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.set_min_width(r.width());
            line_right(ui, r, t.border);
            // The head: its title and ＋ (File › New).
            let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
            panel_head(ui, head, tw!("Drawing Set"), "", 1, &t);
            if head_button(ui, wid("set-new", ()), head, 0, "plus", tw!("New drawing"), &t).clicked() {
                crate::menus::invoke(app, "file.newDialog", json!({}));
            }
            // The open drawings.
            let cap = head.bottom() + 10.0 + 7.0;
            list_caption(ui, r.left() + 12.0, r.right() - 12.0, cap, tw!("Open Drawings"), &t);
            let foot = footer(app, ui, r, &t);
            let avail_h = (foot.top() - 4.0 - (cap + 9.0)).max(0.0);
            let names: Vec<String> = app.session.documents().iter().map(|d| d.title()).collect();
            let open_list_h = ((names.len().clamp(2, 3) as f32) * ROW_H + 2.0).min(avail_h);
            let rows = Rect::from_min_max(pos2(r.left(), cap + 9.0), pos2(r.right() - 1.0, cap + 9.0 + open_list_h));
            let current = app.session.active_index().filter(|_| !crate::menus::home_showing(app));
            let mut open = None;
            let mut list = ui.new_child(egui::UiBuilder::new().max_rect(rows).layout(egui::Layout::top_down(egui::Align::Min)));
            egui::ScrollArea::vertical().id_salt(wid("set-scroll", ())).auto_shrink([false, false]).show(&mut list, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for (i, name) in names.iter().enumerate() {
                    let (_, row) = ui.allocate_space(vec2(ui.available_width(), ROW_H));
                    let row = Rect::from_min_max(pos2(row.left() + 6.0, row.top()), pos2(row.right() - 6.0, row.bottom()));
                    let resp = ui.interact(row, wid("set-row", i), Sense::click());
                    let on = current == Some(i);
                    if on {
                        ui.painter().rect_filled(row, CornerRadius::same(4), t.row_selected);
                    } else if resp.hovered() {
                        ui.painter().rect_filled(row, CornerRadius::same(4), t.hover);
                    }
                    crate::icons::paint(
                        ui,
                        "tw-plan",
                        Rect::from_center_size(pos2(row.left() + 6.0 + 7.0, row.center().y), vec2(14.0, 14.0)),
                        t.icon,
                    );
                    ui.painter().with_clip_rect(row).text(pos2(row.left() + 26.0, row.center().y), Align2::LEFT_CENTER, name, font(12.0), t.text);
                    if resp.on_hover_text(name).clicked() {
                        open = Some(i);
                    }
                }
            });
            if let Some(i) = open {
                app.ui.home = None;
                if let Err(e) = app.run("document.activate", json!({ "index": i })) {
                    app.status(e);
                }
            }

            // Always-visible Scrapboard Palette (소스판 팔레트) right below Open Drawings.
            let scrap_top = rows.bottom() + 4.0;
            if foot.top() - 4.0 > scrap_top + 28.0 {
                let scrap_rect = Rect::from_min_max(pos2(r.left(), scrap_top), pos2(r.right() - 1.0, foot.top() - 4.0));
                line_above(ui, scrap_rect, t.border);
                show_scrapboard_palette(app, ui, scrap_rect, &t);
            }
        });
}

/// Always-visible field scrapboard palette (소스판 팔레트): 1-click access to the 12 Korean field
/// standard rules, scale bars (1/10–1/200), section markers (A–A′ / B–B′ with red ticks), datum
/// markers (┣ 93.0m), north arrows, Roman numeral stratigraphy & 7pt descriptions, and stamps.
fn show_scrapboard_palette(app: &mut VectorcraftApp, ui: &mut Ui, area: Rect, t: &Tokens) {
    let has_doc = app.session.active().is_some();
    let inner = area.shrink2(vec2(0.0, 3.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
    child.set_clip_rect(inner);
    egui::ScrollArea::vertical().id_salt(wid("scrap-scroll", ())).auto_shrink([false, false]).show(&mut child, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        let w = (area.width() - 16.0).max(100.0);
        let left_pad = 8.0;

        let section_head = |ui: &mut Ui, title: &str| {
            let (_, hr) = ui.allocate_space(vec2(ui.available_width(), 18.0));
            let cy = hr.center().y + 1.0;
            list_caption(ui, hr.left() + left_pad, hr.right() - left_pad, cy, title, t);
        };

        let grid_btn = |ui: &Ui, id: egui::Id, r: Rect, label: &str, tip: &str, accent: bool| -> bool {
            let resp = ui.interact(r, id, if has_doc { Sense::click() } else { Sense::hover() });
            let fill = if !has_doc {
                t.panel_darker
            } else if resp.hovered() {
                t.hover
            } else if accent {
                t.row_selected
            } else {
                t.input
            };
            let edge = if accent && has_doc { t.ring } else { t.input_border };
            ui.painter().rect(r, CornerRadius::same(5), fill, Stroke::new(1.0, edge), StrokeKind::Inside);
            let col = if has_doc { t.text } else { t.text_disabled };
            ui.painter().with_clip_rect(r.shrink(2.0)).text(r.center(), Align2::CENTER_CENTER, label, font(10.0), col);
            resp.on_hover_text(tip).clicked() && has_doc
        };

        // 1. 현장 표준 선 규칙 (12종 · 6p/7p 기준)
        section_head(ui, "소스판 · 표준 선 규칙");
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            let btn_r = Rect::from_min_size(pos2(row_r.left() + left_pad, row_r.top()), vec2(w, 21.0));
            if grid_btn(
                ui,
                wid("scrap-preset", "all12"),
                btn_r,
                "★ 현장 실무 표준 12종 레이어 일괄 추가",
                "윤고딕 6p·7p 기준 현장 실무 표준 12종 레이어를 한 번에 추가합니다",
                true,
            ) {
                let _ = app.run("legend.addFieldPresets", json!({}));
            }
        }
        let preset_pairs: [[(&str, &str, &str, &str); 2]; 6] = [
            [
                ("p1", "평면외곽 0.6", "평면외곽선", "평면외곽선 (0.6 pt 실선) 추가 및 선택"),
                ("p2", "내선(바닥) 0.3", "내선(바닥)", "내선(바닥) (0.3 pt 실선) 추가 및 선택"),
            ],
            [
                ("p3", "이중굴광 0.4파", "이중굴광-가운데선", "이중굴광-가운데선 (0.4 pt 파선) 추가 및 선택"),
                ("p4", "단면선 0.8", "단면선", "단면선 (0.8 pt 실선) 추가 및 선택"),
            ],
            [
                ("p5", "석곽외곽 0.5", "석곽-평면외곽선", "석곽-평면외곽선 (0.5 pt 실선) 추가 및 선택"),
                ("p6", "석곽내선 0.2", "석곽-내선", "석곽-내선 (0.2 pt 실선) 추가 및 선택"),
            ],
            [
                ("p7", "유물외곽 0.3", "유물-외곽선", "유물-외곽선 (0.3 pt 실선) 추가 및 선택"),
                ("p8", "유물내선 0.2", "유물-내선", "유물-내선 (0.2 pt 실선) 추가 및 선택"),
            ],
            [
                ("p9", "깨진면 0.15", "석곽·유물 깨진면", "석곽·유물 깨진면 (0.15 pt 실선) 추가 및 선택"),
                ("p10", "단면빗금 0.2", "단면빗금", "단면빗금 (0.2 pt 실선) 추가 및 선택"),
            ],
            [
                ("p11", "해발선 0.3", "해발선", "해발선 (0.3 pt 실선) 추가 및 선택"),
                ("p12", "해발0.5 0.5", "해발 0.5단위 해발선", "해발 0.5단위 해발선 (0.5 pt 실선) 추가 및 선택"),
            ],
        ];
        let col2_w = (w - 4.0) / 2.0;
        for pair in preset_pairs {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, label, target_name, tip)) in pair.into_iter().enumerate() {
                let x = row_r.left() + left_pad + ci as f32 * (col2_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col2_w, 21.0));
                if grid_btn(ui, wid("scrap-preset", key), btn_r, label, tip, false) {
                    let _ = app.run("legend.addFieldPresets", json!({ "name": target_name }));
                }
            }
        }

        // 2. 축척자 (1/10 ~ 1/200 · 윤고딕 6p 규격)
        section_head(ui, "축척자 (1/10~1/200 · 6p)");
        let col3_w = (w - 8.0) / 3.0;
        let scale_style_id = wid("scrap-scale-style", ());
        let mut scale_style = ui.data(|d| d.get_temp::<String>(scale_style_id).unwrap_or_else(|| "report".to_string()));
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (k, lbl, tip)) in [
                ("report", "막대형 축척", "흑백 교차 막대형 축척자 스타일 선택"),
                ("parallel", "평행선형 축척", "소스판식 평행선 눈금형 축척자 스타일 선택"),
            ]
            .into_iter()
            .enumerate()
            {
                let x = row_r.left() + left_pad + ci as f32 * (col2_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col2_w, 21.0));
                if grid_btn(ui, wid("scrap-scstyle", k), btn_r, lbl, tip, scale_style == k) {
                    scale_style = k.to_string();
                    ui.data_mut(|d| d.insert_temp(scale_style_id, scale_style.clone()));
                }
            }
        }
        for row_scales in [[10_u32, 20, 30], [40, 50, 60], [80, 100, 200]] {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, denom) in row_scales.into_iter().enumerate() {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                let lbl = format!("1/{denom}");
                let tip = format!("1/{denom} 표준 축척자 배치 및 도면 축척 1:{denom} 설정");
                if grid_btn(ui, wid("scrap-scale", denom), btn_r, &lbl, &tip, false) {
                    let _ = app.run("doc.setScale", json!({ "scale": f64::from(denom), "epsg": 5186 }));
                    let _ = app.run("trowel.insertScaleBar", json!({ "scale": denom, "style": scale_style }));
                }
            }
        }

        // 3. 단면선 (■ 적색 기준점 + ∠) & 해발 기준점 (┣ 93.0m)
        section_head(ui, "단면선 · 해발 기준점 (■ 적색)");
        for row_sec in [
            [
                ("ah", "A–A′ 수평", "cut", "horizontal", "A", "수평 단면선 A–A′ (적색 기준점 ■ + 시선 화살표) 배치"),
                ("bh", "B–B′ 수평", "cut", "horizontal", "B", "수평 단면선 B–B′ 배치"),
                ("ch", "C–C′ 수평", "cut", "horizontal", "C", "수평 단면선 C–C′ 배치"),
            ],
            [
                ("av", "A–A′ 수직", "cut", "vertical", "A", "수직 단면선 A–A′ 배치"),
                ("dh", "해발 93.0m", "datum", "horizontal", "A", "수평 해발 기준점 (적색 틱 ┣ + 93.0m) 배치"),
                ("dv", "수직 93.0m", "datum", "vertical", "A", "수직 해발 기준점 (적색 틱 ┳ + 93.0m) 배치"),
            ],
        ] {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, lbl, kind, orient, letter, tip)) in row_sec.into_iter().enumerate() {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                if grid_btn(ui, wid("scrap-sec", key), btn_r, lbl, tip, false) {
                    let _ =
                        app.run("trowel.insertSectionMarker", json!({ "kind": kind, "orientation": orient, "label": letter, "elevationM": 93.0 }));
                }
            }
        }
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            let b1 = Rect::from_min_size(pos2(row_r.left() + left_pad, row_r.top()), vec2(col2_w, 21.0));
            let b2 = Rect::from_min_size(pos2(row_r.left() + left_pad + col2_w + 4.0, row_r.top()), vec2(col2_w, 21.0));
            if grid_btn(
                ui,
                wid("scrap-elev", "below"),
                b1,
                "단면 아래 해발선",
                "단면선 아래 지면 영역에 50cm/10cm 해발선(우측 주기 + 적색 기준점) 자동 생성",
                false,
            ) {
                let _ = app.run(
                    "elevation.create",
                    json!({
                        "datumM": 93.0,
                        "minorWeightPt": 0.3,
                        "majorWeightPt": 0.5,
                        "labelFontSizePt": 6.0,
                        "labelPlacement": "right",
                        "fillBelowProfile": true,
                        "redDatumTick": true,
                    }),
                );
            }
            if grid_btn(ui, wid("scrap-elev", "open"), b2, "빈 곳 해발선 생성", "단면 위 굴광 빈 공간에 50cm/10cm 해발선 자동 생성", false)
            {
                let _ = app.run(
                    "elevation.create",
                    json!({
                        "datumM": 93.0,
                        "minorWeightPt": 0.2,
                        "majorWeightPt": 0.5,
                        "labelPlacement": "left",
                        "fillBelowProfile": false,
                    }),
                );
            }
        }

        // 4. 방위표 · 토층(Ⅰ~Ⅹ) · 현장 스탬프
        section_head(ui, "방위 · 토층(Ⅰ~Ⅹ) · 스탬프");
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, lbl, deg, tip)) in [
                ("n0", "방위 N 0°", 0.0, "정북(0°) 방위표 배치"),
                ("n30", "방위 30°", 30.0, "30° 회전 방위표 배치"),
                ("nm30", "방위 -30°", -30.0, "-30° 회전 방위표 배치"),
            ]
            .into_iter()
            .enumerate()
            {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                if grid_btn(ui, wid("scrap-north", key), btn_r, lbl, tip, false) {
                    let _ = app.run("trowel.insertNorthArrow", json!({ "rotationDeg": deg }));
                }
            }
        }
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, lbl, tip, act_kind)) in [
                ("roman", "로마 Ⅰ·Ⅱ·Ⅲ", "토층 번호를 로마 숫자(Ⅰ, Ⅱ, Ⅲ…) 6p 규격으로 도면에 배치", 0_u8),
                ("circled", "원문자 ①·②", "토층 번호를 원문자(①, ②, ③…)로 도면에 배치", 1_u8),
                ("table7p", "7p 토층설명", "윤고딕 7p 일렬 배치 방식의 표준 토층 설명 블록을 도면에 삽입", 2_u8),
            ]
            .into_iter()
            .enumerate()
            {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                if grid_btn(ui, wid("scrap-strat", key), btn_r, lbl, tip, false) {
                    match act_kind {
                        0 => {
                            for n in 1..=5_u32 {
                                let _ = app.run(
                                    "stratigraphy.circledNumber",
                                    json!({ "at": [160.0 + f64::from(n) * 20.0, 200.0], "number": n, "style": "roman_ascii", "fontSizePt": 6.0 }),
                                );
                            }
                        }
                        1 => {
                            for n in 1..=5_u32 {
                                let _ = app.run(
                                    "stratigraphy.circledNumber",
                                    json!({ "at": [160.0 + f64::from(n) * 20.0, 224.0], "number": n, "style": "circled", "fontSizePt": 6.0 }),
                                );
                            }
                        }
                        _ => {
                            let _ = app.run("stratigraphy.generateTable", json!({ "at": [120, 440], "format": "inline", "fontSizePt": 7.0 }));
                        }
                    }
                }
            }
        }
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, lbl, tip, stamp_kind)) in [
                ("disturb", "교란 (6p)", "교란 지점(점선 타원 + 6p 주기) 스탬프 배치", "disturbance"),
                ("limit", "조사경계선", "조사경계 (일점쇄선 + 6p 주기) 스탬프 배치", "excavation_limit"),
                ("grid_lbl", "격자기호 A1", "격자 기호(A1~C3) 스탬프 배치", "grid_labels"),
            ]
            .into_iter()
            .enumerate()
            {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                if grid_btn(ui, wid("scrap-stamp", key), btn_r, lbl, tip, false) {
                    let _ = app.run("trowel.insertFieldStamp", json!({ "kind": stamp_kind }));
                }
            }
        }
    });
}

/// The set's footer at the bottom of `r`: the current drawing's size, its scale and coordinate
/// system (J006). Returns its rect.
fn footer(app: &mut VectorcraftApp, ui: &Ui, r: Rect, t: &Tokens) -> Rect {
    let line = 11.0 * 1.6;
    let foot = Rect::from_min_max(pos2(r.left(), r.bottom() - 8.0 - 3.0 * line - 8.0), r.max);
    line_above(ui, foot, t.border);
    let size = app.session.active().and_then(|d| {
        let ab = d.doc.artboards.get(app.view().map_or(0, |v| v.artboard)).or_else(|| d.doc.artboards.first())?;
        let u = app.session.general_unit();
        Some(format!("{} × {}", u.format(ab.rect.width()), u.format(ab.rect.height())))
    });
    let scale_info = app.session.active().and_then(|d| {
        let cfg = d.doc.scale.as_ref()?;
        Some((format!("축척 1:{:.0}", cfg.scale.denom), format!("좌표계 EPSG:{} (TM)", cfg.crs.epsg)))
    });
    let mut y = foot.top() + 8.0 + line / 2.0;
    if let Some(size) = size {
        let x = text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, format!("{} · ", tw!("This drawing")), font(11.0), t.text_dim).right();
        text(ui, pos2(x, y), Align2::LEFT_CENTER, size, mono(11.0), t.text);
    }
    y += line;
    let soon_top = y - line / 2.0;
    if let Some((scale_str, crs_str)) = scale_info {
        text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, scale_str, mono(11.0), t.text);
        y += line;
        text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, crs_str, font(11.0), t.text_dim);
        y += line;
    } else {
        for l in [tw!("Scale not set"), tw!("Coordinate system not set")] {
            text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, l, font(11.0), t.text_dim);
            y += line;
        }
    }
    let lines = Rect::from_min_max(pos2(foot.left(), soon_top), pos2(foot.right() - 1.0, y - line / 2.0));
    let resp = ui.interact(lines, wid("set-foot", ()), Sense::click()).on_hover_text(tw!("Scale and coordinate system"));
    if resp.clicked() {
        let next_denom = app
            .session
            .active()
            .and_then(|d| d.doc.scale.as_ref().map(|c| c.scale.denom))
            .map(|d| match d as u32 {
                1 => 10.0,
                10 => 20.0,
                20 => 50.0,
                50 => 100.0,
                _ => 20.0,
            })
            .unwrap_or(20.0);
        let _ = app.run("doc.setScale", json!({ "scale": next_denom, "epsg": 5186 }));
    }
    foot
}
