//! The drawing set: the open drawings, the current one marked; a click shows one (from the Home
//! screen too) and ＋ starts a new one. Under them, until the element box (design 3.0 D5) takes
//! its place, the set of one-click drawing elements. The drawing's scale and coordinate system
//! have their place in the status bar.

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
            let avail_h = (r.bottom() - 4.0 - (cap + 9.0)).max(0.0);
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
            if r.bottom() - 4.0 > scrap_top + 28.0 {
                let scrap_rect = Rect::from_min_max(pos2(r.left(), scrap_top), pos2(r.right() - 1.0, r.bottom() - 4.0));
                line_above(ui, scrap_rect, t.border);
                show_scrapboard_palette(app, ui, scrap_rect, &t);
            }
        });
}

/// Always-visible field symbol palette (도면 기호 소스판): 1-click placement of scale bars
/// (1/10–1/200), section markers (A–A′ / B–B′ with red datum blocks), elevation datum & level
/// lines, north arrows, Roman numeral stratigraphy & 7pt descriptions, field stamps, and a
/// compact foldable quick-setup bar for the 12 field layer rules.
fn show_scrapboard_palette(app: &mut VectorcraftApp, ui: &mut Ui, area: Rect, t: &Tokens) {
    let has_doc = app.session.active().is_some();
    let inner = area.shrink2(vec2(0.0, 3.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
    child.set_clip_rect(inner);
    egui::ScrollArea::vertical().id_salt(wid("scrap-scroll", ())).auto_shrink([false, false]).show(&mut child, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        let w = (area.width() - 16.0).max(100.0);
        let left_pad = 8.0;
        let col2_w = (w - 4.0) / 2.0;
        let col3_w = (w - 8.0) / 3.0;

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

        // 1. 축척자 (1/10 ~ 1/200 · 윤고딕 6p 규격)
        section_head(ui, "축척자 (1/10~1/200 · 6p)");
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
                    let at = super::place_symbol_at(app, ui.ctx(), [0.0, 90.0]);
                    let _ = app.run("doc.setScale", json!({ "scale": f64::from(denom), "epsg": 5186 }));
                    let _ = app.run("trowel.insertScaleBar", json!({ "scale": denom, "style": scale_style, "at": at }));
                }
            }
        }

        // 2. 단면선 (■ 적색 기준점 + ∠) & 해발 기준점 (──┃ 93.0m), at the datum set here (it was
        // in the ribbon's Spot Height popup; the tool column's Spot Height uses it too).
        section_head(ui, "단면선 · 해발 기준점 (■ 적색)");
        let mut elev_m = super::datum_elevation_m(ui.ctx());
        ui.horizontal(|ui| {
            ui.add_space(left_pad);
            ui.label(egui::RichText::new(tw!("Datum elevation")).font(font(11.0)).color(t.text_dim));
            ui.add_enabled(has_doc, egui::DragValue::new(&mut elev_m).speed(0.1).range(-100.0..=2000.0).fixed_decimals(1).suffix(" m"));
        });
        ui.data_mut(|d| d.insert_temp(super::datum_elevation_id(), elev_m));
        let datum_label = |orient: &str| match orient {
            "horizontal" => format!("해발 {elev_m:.1}m"),
            _ => format!("수직 {elev_m:.1}m"),
        };
        for row_sec in [
            [
                ("ah", "A–A′ 수평", "cut", "horizontal", "A", "수평 단면선 A–A′ (적색 기준점 ■ + 시선 화살표 ∠) 배치"),
                ("bh", "B–B′ 수평", "cut", "horizontal", "B", "수평 단면선 B–B′ 배치"),
                ("ch", "C–C′ 수평", "cut", "horizontal", "C", "수평 단면선 C–C′ 배치"),
            ],
            [
                ("av", "A–A′ 수직", "cut", "vertical", "A", "수직 단면선 A–A′ 배치"),
                ("dh", "", "datum", "horizontal", "A", "수평 해발 기준점 (적색 수직 바 ──┃ + 기준 해발) 배치"),
                ("dv", "", "datum", "vertical", "A", "수직 해발 기준점 (적색 수평 바 ┳ + 기준 해발) 배치"),
            ],
        ] {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, lbl, kind, orient, letter, tip)) in row_sec.into_iter().enumerate() {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                let lbl = if kind == "datum" { datum_label(orient) } else { lbl.to_string() };
                if grid_btn(ui, wid("scrap-sec", key), btn_r, &lbl, tip, false) {
                    let at = super::place_symbol_at(app, ui.ctx(), [0.0, 0.0]);
                    let _ = app.run(
                        "trowel.insertSectionMarker",
                        json!({ "kind": kind, "orientation": orient, "label": letter, "elevationM": elev_m, "at": at }),
                    );
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
                "단면선 아래 지면 영역에 50cm/10cm 연속 해발선(우측 주기 + 적색 기준점) 자동 생성",
                false,
            ) {
                // Near the middle of the view when there's no section to follow.
                let at = super::place_symbol_at(app, ui.ctx(), [0.0, 0.0]);
                let _ = app.run(
                    "elevation.create",
                    json!({
                        "at": at,
                        "datumM": elev_m,
                        "minorWeightPt": 0.3,
                        "majorWeightPt": 0.5,
                        "labelFontSizePt": 6.0,
                        "labelPlacement": "right",
                        "fillBelowProfile": true,
                        "redDatumTick": true,
                    }),
                );
            }
            if grid_btn(ui, wid("scrap-elev", "open"), b2, "빈 곳 해발선 생성", "단면 위 굴광 빈 공간에 50cm/10cm 연속 해발선 자동 생성", false)
            {
                // Near the middle of the view when there's no section to follow.
                let at = super::place_symbol_at(app, ui.ctx(), [0.0, 0.0]);
                let _ = app.run(
                    "elevation.create",
                    json!({
                        "at": at,
                        "datumM": elev_m,
                        "minorWeightPt": 0.2,
                        "majorWeightPt": 0.5,
                        "labelPlacement": "left",
                        "fillBelowProfile": false,
                    }),
                );
            }
        }

        // 3. 방위표 · 토층(Ⅰ~Ⅹ) · 현장 스탬프
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
                    let at = super::place_symbol_at(app, ui.ctx(), [90.0, -70.0]);
                    let _ = app.run("trowel.insertNorthArrow", json!({ "rotationDeg": deg, "at": at }));
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
                            let base = super::place_symbol_at(app, ui.ctx(), [-40.0, -10.0]);
                            for n in 1..=5_u32 {
                                let _ = app.run(
                                    "stratigraphy.circledNumber",
                                    json!({
                                        "at": [base[0] + f64::from(n - 1) * 20.0, base[1]],
                                        "number": n,
                                        "style": "roman_ascii",
                                        "fontSizePt": 6.0,
                                    }),
                                );
                            }
                        }
                        1 => {
                            let base = super::place_symbol_at(app, ui.ctx(), [-40.0, 14.0]);
                            for n in 1..=5_u32 {
                                let _ = app.run(
                                    "stratigraphy.circledNumber",
                                    json!({
                                        "at": [base[0] + f64::from(n - 1) * 20.0, base[1]],
                                        "number": n,
                                        "style": "circled",
                                        "fontSizePt": 6.0,
                                    }),
                                );
                            }
                        }
                        _ => {
                            let at = super::place_symbol_at(app, ui.ctx(), [-80.0, 60.0]);
                            let _ = app.run("stratigraphy.generateTable", json!({ "at": at, "format": "inline", "fontSizePt": 7.0 }));
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
                    let at = super::place_symbol_at(app, ui.ctx(), [0.0, 20.0]);
                    let _ = app.run("trowel.insertFieldStamp", json!({ "kind": stamp_kind, "at": at }));
                }
            }
        }

        // 4. 표준 레이어 빠른 설정 (우측 범례·레이어 창 연동, 기본 2줄 요약 + 펼치기)
        section_head(ui, "표준 레이어 빠른 생성");
        let expand_id = wid("scrap-preset-expanded", ());
        let mut expanded = ui.data(|d| d.get_temp::<bool>(expand_id).unwrap_or(false));
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            let main_w = (w - 48.0).max(60.0);
            let btn_r = Rect::from_min_size(pos2(row_r.left() + left_pad, row_r.top()), vec2(main_w, 21.0));
            let fold_r = Rect::from_min_size(pos2(btn_r.right() + 4.0, row_r.top()), vec2(44.0, 21.0));
            if grid_btn(
                ui,
                wid("scrap-preset", "all12"),
                btn_r,
                "★ 실무 12종 일괄 추가",
                "윤고딕 6p·7p 기준 현장 실무 표준 12종 레이어를 우측 범례·레이어 창에 한 번에 추가합니다",
                true,
            ) {
                let _ = app.run("legend.addFieldPresets", json!({}));
            }
            let fold_lbl = if expanded { "접기 ▴" } else { "더보기 ▾" };
            if grid_btn(ui, wid("scrap-preset-fold", ()), fold_r, fold_lbl, "개별 표준 레이어 12종 목록 펼치기/접기", false) {
                expanded = !expanded;
                ui.data_mut(|d| d.insert_temp(expand_id, expanded));
            }
        }
        let core_row = [
            ("p1", "평면외곽", "평면외곽선", "평면외곽선 (0.6 pt 실선) 레이어 추가 및 선택"),
            ("p2", "내선(바닥)", "내선(바닥)", "내선(바닥) (0.3 pt 실선) 레이어 추가 및 선택"),
            ("p3", "이중굴광", "이중굴광시-가운데선", "이중굴광시-가운데선 (0.4 pt 파선) 레이어 추가 및 선택"),
        ];
        {
            let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
            for (ci, (key, label, target_name, tip)) in core_row.into_iter().enumerate() {
                let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                if grid_btn(ui, wid("scrap-preset", key), btn_r, label, tip, false) {
                    let _ = app.run("legend.addFieldPresets", json!({ "name": target_name }));
                }
            }
        }
        if expanded {
            let extra_rows: [[(&str, &str, &str, &str); 3]; 3] = [
                [
                    ("p4", "단면선", "단면선", "단면선 (0.8 pt 실선) 레이어 추가 및 선택"),
                    ("p5", "석곽외곽", "석곽-평면외곽선", "석곽-평면외곽선 (0.5 pt 실선) 레이어 추가 및 선택"),
                    ("p6", "석곽내선", "석곽-내선", "석곽-내선 (0.2 pt 실선) 레이어 추가 및 선택"),
                ],
                [
                    ("p7", "유물외곽", "석곽-유물 토기외곽·금속", "석곽-유물 토기외곽·금속 (0.3 pt 실선) 레이어 추가 및 선택"),
                    ("p8", "유물내선", "석곽-유물 내선", "석곽-유물 내선 (0.2 pt 실선) 레이어 추가 및 선택"),
                    ("p9", "깨진면", "석곽-유물 깨진면", "석곽-유물 깨진면 (0.15 pt 실선) 레이어 추가 및 선택"),
                ],
                [
                    ("p10", "단면빗금", "단면빗금", "단면빗금 (0.2 pt 실선) 레이어 추가 및 선택"),
                    ("p11", "해발선", "해발선", "해발선 (0.3 pt 실선) 레이어 추가 및 선택"),
                    ("p12", "0.5m 주해발", "해발선 0.5단위", "해발선 0.5단위 (0.5 pt 실선) 레이어 추가 및 선택"),
                ],
            ];
            for row_presets in extra_rows {
                let (_, row_r) = ui.allocate_space(vec2(ui.available_width(), 22.0));
                for (ci, (key, label, target_name, tip)) in row_presets.into_iter().enumerate() {
                    let x = row_r.left() + left_pad + ci as f32 * (col3_w + 4.0);
                    let btn_r = Rect::from_min_size(pos2(x, row_r.top()), vec2(col3_w, 21.0));
                    if grid_btn(ui, wid("scrap-preset", key), btn_r, label, tip, false) {
                        let _ = app.run("legend.addFieldPresets", json!({ "name": target_name }));
                    }
                }
            }
        }
    });
}
