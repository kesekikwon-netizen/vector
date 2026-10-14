//! Trowel Start Screen (`홈` tab) — archaeological project hub (design 2.1 §2).
//!
//! Layout:
//! 1. Hero:
//!    - Left: Brand logo + "Trowel 발굴 도면" + Buttons [＋ 새 도면], [도면집 열기], [Strata 조사에서 시작]
//!    - Right: "이어서 작업" card with primary Clay button [이어서 그리기]
//! 2. Homecols:
//!    - Left: Recent Drawing Books table ("최근 도면집") with right-click context menu "목록에서 빼기"
//!    - Right: 7-stage Archaeological Workflow rail ("작업 순서")
//! 3. Archaeological Templates ("새 도면 견본"):
//!    - 개별유구 평·단면도 (1:20 A3)
//!    - 토층 단면도 (1:20 A3)
//!    - 유물 실측도 (1:1 A4)
//!    - 유구배치도 (1:100 A3)
//!    - 보고서 도판 (A4)
//! 4. Connection Summary Bar (`conn`):
//!    - Strata GPKG link, desca sections, survey control points, underlay scans, report fonts.

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use super::book::RecentDrawingBooks;
use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

/// Shows the Trowel Start Screen.
pub(crate) fn show(app: &mut VectorcraftApp, ui: &mut Ui, below: Rect) {
    let t = Tokens::get(ui.ctx());

    // Fill the background of the Start Screen
    ui.painter().rect_filled(below, 0.0, t.pasteboard);

    let mut action_cmd = None;

    // Load recent drawing books from egui temp data or fallback to defaults
    let recents_id = egui::Id::new("trowel_recent_books");
    let mut recents = ui.data_mut(|d| d.get_temp_mut_or_default::<RecentDrawingBooks>(recents_id).clone());
    if recents.books.is_empty() {
        recents = RecentDrawingBooks::new();
    }

    egui::ScrollArea::vertical()
        .id_salt(wid("home-scroll", ()))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let width = ui.available_width().min(1500.0);
            let margin = ((ui.available_width() - width) / 2.0).max(16.0);

            ui.horizontal(|ui| {
                ui.add_space(margin);
                ui.vertical(|ui| {
                    ui.set_max_width(width);
                    ui.add_space(20.0);

                    // 1. Hero Section
                    if let Some(cmd) = render_hero(ui, &t) {
                        action_cmd = Some(cmd);
                    }
                    ui.add_space(24.0);

                    // 2. Home Columns (Recent Books + Workflow Rail)
                    if let Some(cmd) = render_homecols(ui, &mut recents, &t) {
                        action_cmd = Some(cmd);
                    }
                    ui.add_space(28.0);

                    // 3. Archaeological Templates
                    if let Some(cmd) = render_templates(ui, &t) {
                        action_cmd = Some(cmd);
                    }
                    ui.add_space(24.0);

                    // 4. Connection Summary Bar
                    render_connection_bar(ui, &t);
                    ui.add_space(24.0);
                });
                ui.add_space(margin);
            });
        });

    // Save updated recents back to temp storage
    ui.data_mut(|d| d.insert_temp(recents_id, recents));

    // Execute any command triggered from Start Screen
    if let Some((cmd, params)) = action_cmd {
        app.ui.home = None; // Switch away from home screen
        let _ = app.run(cmd, params);
    }
}

/// Renders the Hero section (Brand + Quick Actions & Resume Work card).
fn render_hero(ui: &mut Ui, t: &Tokens) -> Option<(&'static str, serde_json::Value)> {
    let mut dispatched = None;
    let full_w = ui.available_width();
    let r_card_w = 480.0f32.min(full_w * 0.42);
    let l_w = full_w - r_card_w - 32.0;

    let start_pos = ui.cursor().min;
    let sep_stroke = Stroke::new(1.0, t.text);

    // Top border line
    ui.painter().hline(start_pos.x..=(start_pos.x + full_w), start_pos.y, sep_stroke);
    ui.add_space(18.0);

    ui.horizontal(|ui| {
        // Left Column: Brand Logo + Slogan + Buttons
        ui.vertical(|ui| {
            ui.set_width(l_w);

            // Brand line: Logo icon + "Trowel" + rule + "발굴 도면"
            ui.horizontal(|ui| {
                let logo_size = vec2(40.0, 40.0);
                let (logo_rect, _) = ui.allocate_exact_size(logo_size, Sense::hover());
                draw_trowel_logo(ui, logo_rect, t);

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Trowel").font(bold(26.0)).color(t.text));

                // Vertical separator
                let rule_h = 24.0;
                let c_y = logo_rect.center().y;
                ui.painter().vline(ui.cursor().min.x + 8.0, (c_y - rule_h / 2.0)..=(c_y + rule_h / 2.0), Stroke::new(1.0, t.border));
                ui.add_space(16.0);

                ui.label(egui::RichText::new("발굴 도면").font(font(15.0)).color(t.text_dim));
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("도면집을 이어서 열거나, 새 도면을 시작하세요.").font(font(15.0)).color(t.text));
            ui.add_space(12.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let r_new = action_button(ui, wid("btn", "new"), "＋ 새 도면", false, t);
                if r_new.clicked() {
                    dispatched = Some(("trowel.newPlan", json!({})));
                }

                ui.add_space(6.0);
                let r_open = action_button(ui, wid("btn", "open_book"), "도면집 열기", false, t);
                if r_open.clicked() {
                    dispatched = Some(("file.open", json!({})));
                }

                ui.add_space(6.0);
                let r_strata = action_button(ui, wid("btn", "strata_start"), "Strata 조사에서 시작", false, t);
                if r_strata.clicked() {
                    dispatched = Some(("file.open", json!({})));
                }
            });
        });

        // Vertical divider between hero columns
        let cy_top = start_pos.y + 12.0;
        let cy_bottom = ui.cursor().max.y + 8.0;
        ui.painter().vline(ui.cursor().min.x + 12.0, cy_top..=cy_bottom, Stroke::new(1.0, t.border));
        ui.add_space(24.0);

        // Right Column: Resume Work Box ("이어서 작업")
        ui.vertical(|ui| {
            ui.set_width(r_card_w);
            ui.label(egui::RichText::new("이어서 작업").font(font(11.0)).color(t.text_dim));
            ui.add_space(2.0);
            ui.label(egui::RichText::new("1호 주거지 평면").font(bold(19.0)).color(t.text));
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("광령리1 도면집 · 1:20 · 오늘 07:04에 닫음\n3호 수혈을 그리던 중이었습니다.")
                    .font(font(12.0))
                    .color(t.text_dim),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                // Clay Button: [이어서 그리기]
                let r_resume = action_button(ui, wid("btn", "resume"), "이어서 그리기", true, t);
                if r_resume.clicked() {
                    dispatched = Some(("trowel.newPlan", json!({ "name": "1호 주거지 평면" })));
                }

                ui.add_space(8.0);
                let _ = action_button(ui, wid("btn", "view_book"), "도면집 보기", false, t);
            });
        });
    });

    ui.add_space(18.0);
    // Bottom border line
    let cur_y = ui.cursor().min.y;
    ui.painter().hline(start_pos.x..=(start_pos.x + full_w), cur_y, sep_stroke);
    ui.add_space(6.0);

    dispatched
}

/// Renders the Recent Drawing Books table and the 7-stage Workflow rail.
fn render_homecols(ui: &mut Ui, recents: &mut RecentDrawingBooks, t: &Tokens) -> Option<(&'static str, serde_json::Value)> {
    let mut dispatched = None;
    let full_w = ui.available_width();
    let rail_w = 340.0f32.min(full_w * 0.36);
    let table_w = full_w - rail_w - 32.0;

    ui.horizontal(|ui| {
        // Left Column: Recent Drawing Books Table
        ui.vertical(|ui| {
            ui.set_width(table_w);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("최근 도면집").font(bold(15.0)).color(t.text));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(format!("{}개", recents.books.len())).font(font(11.0)).color(t.text_dim));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("오른쪽 클릭: 목록에서 빼기").font(font(11.0)).color(t.text_dim));
                });
            });
            ui.add_space(8.0);

            // Table rendering
            let mut remove_index = None;
            render_books_table(ui, recents, &mut remove_index, &mut dispatched, table_w, t);

            if let Some(idx) = remove_index {
                recents.remove_at(idx);
            }
        });

        ui.add_space(32.0);

        // Right Column: Workflow Rail ("작업 순서")
        ui.vertical(|ui| {
            ui.set_width(rail_w);
            render_workflow_rail(ui, t);
        });
    });

    dispatched
}

/// Renders the Recent Drawing Books table with status, counts, crs, and right-click context menu.
fn render_books_table(
    ui: &mut Ui,
    recents: &RecentDrawingBooks,
    remove_index: &mut Option<usize>,
    dispatched: &mut Option<(&'static str, serde_json::Value)>,
    width: f32,
    t: &Tokens,
) {
    let table_top = ui.cursor().min;
    let header_h = 28.0;

    // Header line above
    ui.painter().hline(table_top.x..=(table_top.x + width), table_top.y, Stroke::new(1.0, t.text));

    // Column widths
    let col_book = width * 0.28;
    let col_strata = width * 0.26;
    let col_num = 32.0;
    let col_crs = 100.0;
    let col_date = width - (col_book + col_strata + col_num * 4.0 + col_crs);

    // Draw header row
    let (h_rect, _) = ui.allocate_exact_size(vec2(width, header_h), Sense::hover());
    let cy = h_rect.center().y;
    let mut x = h_rect.left();

    text(ui, pos2(x + 4.0, cy), Align2::LEFT_CENTER, "도면집", font(11.0), t.text_dim);
    x += col_book;
    text(ui, pos2(x + 4.0, cy), Align2::LEFT_CENTER, "Strata 조사", font(11.0), t.text_dim);
    x += col_strata;
    text(ui, pos2(x + col_num - 4.0, cy), Align2::RIGHT_CENTER, "평면", font(11.0), t.text_dim);
    x += col_num;
    text(ui, pos2(x + col_num - 4.0, cy), Align2::RIGHT_CENTER, "단면", font(11.0), t.text_dim);
    x += col_num;
    text(ui, pos2(x + col_num - 4.0, cy), Align2::RIGHT_CENTER, "유물", font(11.0), t.text_dim);
    x += col_num;
    text(ui, pos2(x + col_num - 4.0, cy), Align2::RIGHT_CENTER, "도판", font(11.0), t.text_dim);
    x += col_num;
    text(ui, pos2(x + 4.0, cy), Align2::LEFT_CENTER, "좌표계", font(11.0), t.text_dim);
    x += col_crs;
    text(ui, pos2(x + col_date - 4.0, cy), Align2::RIGHT_CENTER, "마지막 열림", font(11.0), t.text_dim);

    // Header bottom border
    ui.painter().hline(table_top.x..=(table_top.x + width), h_rect.bottom(), Stroke::new(1.0, t.border));

    // Draw rows
    for (i, book) in recents.books.iter().enumerate() {
        let row_h = 44.0;
        let (row_rect, resp) = ui.allocate_exact_size(vec2(width, row_h), Sense::click());

        if resp.hovered() {
            ui.painter().rect_filled(row_rect, 0.0, t.row_selected);
        }

        // Context menu: "목록에서 빼기"
        resp.context_menu(|ui| {
            if ui.button("목록에서 빼기").clicked() {
                *remove_index = Some(i);
                ui.close();
            }
        });

        if resp.clicked() {
            *dispatched = Some(("trowel.newPlan", json!({ "name": book.name })));
        }

        let r_cy = row_rect.center().y;
        let mut rx = row_rect.left();

        // 1. 도면집 name & path
        text(ui, pos2(rx + 4.0, r_cy - 8.0), Align2::LEFT_CENTER, &book.name, bold(14.0), t.text);
        let path_str = book.drawing_book_dir.to_string_lossy();
        text(ui, pos2(rx + 4.0, r_cy + 9.0), Align2::LEFT_CENTER, path_str, font(11.0), t.text_dim);
        rx += col_book;

        // 2. Strata connection status
        match &book.strata_status {
            super::book::StrataGpkgStatus::Connected { filename, .. } => {
                text(ui, pos2(rx + 4.0, r_cy - 8.0), Align2::LEFT_CENTER, filename, font(12.0), t.text);
                text(ui, pos2(rx + 4.0, r_cy + 9.0), Align2::LEFT_CENTER, "읽기만 하는 연결", font(11.0), t.text_dim);
            }
            super::book::StrataGpkgStatus::Disconnected => {
                text(ui, pos2(rx + 4.0, r_cy), Align2::LEFT_CENTER, "연결 안 함", font(12.0), t.text_dim);
            }
            super::book::StrataGpkgStatus::Missing { filename, .. } => {
                let red = t.error;
                text(ui, pos2(rx + 4.0, r_cy - 8.0), Align2::LEFT_CENTER, "■ 원본 없음", bold(12.0), red);
                let sub = format!("{filename}를 찾지 못했습니다");
                text(ui, pos2(rx + 4.0, r_cy + 9.0), Align2::LEFT_CENTER, sub, font(11.0), t.text_dim);
            }
        }
        rx += col_strata;

        // 3-6. Counts
        text(ui, pos2(rx + col_num - 4.0, r_cy), Align2::RIGHT_CENTER, book.plan_count.to_string(), mono(12.0), t.text);
        rx += col_num;
        text(ui, pos2(rx + col_num - 4.0, r_cy), Align2::RIGHT_CENTER, book.section_count.to_string(), mono(12.0), t.text);
        rx += col_num;
        text(ui, pos2(rx + col_num - 4.0, r_cy), Align2::RIGHT_CENTER, book.finds_count.to_string(), mono(12.0), t.text);
        rx += col_num;
        text(ui, pos2(rx + col_num - 4.0, r_cy), Align2::RIGHT_CENTER, book.plate_count.to_string(), mono(12.0), t.text);
        rx += col_num;

        // 7. CRS
        text(ui, pos2(rx + 4.0, r_cy), Align2::LEFT_CENTER, &book.crs, font(12.0), t.text);
        rx += col_crs;

        // 8. Last opened
        text(ui, pos2(rx + col_date - 4.0, r_cy), Align2::RIGHT_CENTER, &book.last_opened, font(11.0), t.text_dim);

        // Row dividing line
        ui.painter().hline(table_top.x..=(table_top.x + width), row_rect.bottom(), Stroke::new(1.0, t.border.gamma_multiply(0.6)));
    }

    // Bottom table border line
    let cur_y = ui.cursor().min.y;
    ui.painter().hline(table_top.x..=(table_top.x + width), cur_y, Stroke::new(1.0, t.text));
}

/// Renders the 7-stage archaeological survey workflow rail.
fn render_workflow_rail(ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("작업 순서").font(bold(15.0)).color(t.text));
        ui.add_space(6.0);
        ui.label(egui::RichText::new("광령리1 기준").font(font(11.0)).color(t.text_dim));
    });
    ui.add_space(10.0);

    let rail_top = ui.cursor().min;
    let steps = [
        ("1. 자료 받기", "Strata 조사 파일, 측점 38개, 스캔 6장", RailState::Done),
        ("2. 밑그림 맞추기", "스캔 6장 모두 2점 맞춤", RailState::Done),
        ("3. 평면 그리기", "1호 주거지 끝, 3호 수혈 그리는 중", RailState::Next),
        ("4. 단면·토층", "A–A′, B–B′, Tr-1 북벽", RailState::Todo(4)),
        ("5. 유물 실측", "1호 주거지 출토 8점", RailState::Todo(5)),
        ("6. 도판 만들기", "도면 11, 12 초안 있음", RailState::Todo(6)),
        ("7. 내보내기", "보고서 PDF, Strata 제출용 PDF", RailState::Todo(7)),
    ];

    let dot_center_x = rail_top.x + 12.0;

    for (i, (title, desc, state)) in steps.iter().enumerate() {
        let (row_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
        let cy = row_rect.center().y;

        // Draw vertical connecting line between steps
        if i + 1 < steps.len() {
            ui.painter().vline(dot_center_x, cy..=(cy + 32.0), Stroke::new(1.0, t.border));
        }

        // Draw stage marker circle
        let center = pos2(dot_center_x, cy);
        match state {
            RailState::Done => {
                ui.painter().circle(center, 9.0, t.input, Stroke::new(1.0, t.text));
                text(ui, center, Align2::CENTER_CENTER, "✓", bold(10.0), t.text);
            }
            RailState::Next => {
                ui.painter().circle(center, 9.0, t.pasteboard, Stroke::new(1.0, t.border));
                ui.painter().circle_filled(center, 4.0, t.clay);
            }
            RailState::Todo(num) => {
                ui.painter().circle(center, 9.0, t.pasteboard, Stroke::new(1.0, t.border));
                text(ui, center, Align2::CENTER_CENTER, num.to_string(), mono(10.0), t.text_dim);
            }
        }

        // Title and description
        let text_x = dot_center_x + 16.0;
        let t_rect = text(ui, pos2(text_x, cy - 7.0), Align2::LEFT_CENTER, *title, bold(12.0), t.text);

        if matches!(state, RailState::Next) {
            let clay_text = t.clay_text;
            text(ui, pos2(t_rect.right() + 6.0, cy - 7.0), Align2::LEFT_CENTER, "다음", bold(11.0), clay_text);
        }

        text(ui, pos2(text_x, cy + 8.0), Align2::LEFT_CENTER, *desc, font(11.0), t.text_dim);
        ui.add_space(2.0);
    }
}

enum RailState {
    Done,
    Next,
    Todo(u8),
}

/// Renders the 5 archaeological project template cards.
fn render_templates(ui: &mut Ui, t: &Tokens) -> Option<(&'static str, serde_json::Value)> {
    let mut dispatched = None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("새 도면 견본").font(bold(15.0)).color(t.text));
        ui.add_space(6.0);
        ui.label(egui::RichText::new("범례·축척·용지·글자 크기가 미리 맞춰져 있습니다").font(font(11.0)).color(t.text_dim));
    });
    ui.add_space(10.0);

    let templates = [
        ("plan", "개별유구 평·단면도", "1:20 · A3 가로 · 평면용 범례", "trowel.newPlan"),
        ("section", "토층 단면도", "1:20 · 기준선 · 단면용 범례", "trowel.newSection"),
        ("finds", "유물 실측도", "1:1로 그림 · 도판은 1:3", "trowel.newFinds"),
        ("siteplan", "유구배치도", "1:100 · Strata 조사에서", "trowel.newSitePlan"),
        ("plate", "보고서 도판", "A4 세로 · 번호와 캡션", "trowel.newPlate"),
    ];

    let full_w = ui.available_width();
    let gap = 14.0;
    let card_w = (full_w - gap * 4.0) / 5.0;

    ui.horizontal(|ui| {
        for (i, (key, title, subtitle, cmd)) in templates.iter().enumerate() {
            if i > 0 {
                ui.add_space(gap);
            }

            let card_size = vec2(card_w, 140.0);
            let (card_rect, resp) = ui.allocate_exact_size(card_size, Sense::click());

            let is_hovered = resp.hovered();
            let preview_rect = Rect::from_min_size(card_rect.min, vec2(card_w, 88.0));

            // Preview well box
            let well_fill = if is_hovered { t.row_selected } else { t.panel_darker };
            ui.painter().rect(preview_rect, CornerRadius::same(8), well_fill, Stroke::new(1.0, t.border), StrokeKind::Inside);

            // Miniature drawing preview inside well
            let inner_preview = Rect::from_center_size(preview_rect.center(), vec2(card_w * 0.76, 64.0));
            ui.painter().rect_filled(inner_preview, CornerRadius::same(2), t.input);
            ui.painter().rect_stroke(inner_preview, CornerRadius::same(2), Stroke::new(1.0, t.input_border), StrokeKind::Inside);
            draw_template_glyph(ui, inner_preview, key, t);

            // Labels below preview
            let title_y = preview_rect.bottom() + 10.0;
            text(ui, pos2(card_rect.left(), title_y), Align2::LEFT_TOP, *title, bold(13.0), t.text);

            let sub_y = title_y + 18.0;
            text(ui, pos2(card_rect.left(), sub_y), Align2::LEFT_TOP, *subtitle, font(11.0), t.text_dim);

            if resp.clicked() {
                dispatched = Some((*cmd, json!({})));
            }
        }
    });

    dispatched
}

/// Renders the Connection Summary Bar (`conn`).
fn render_connection_bar(ui: &mut Ui, t: &Tokens) {
    let bar_top = ui.cursor().min;
    let width = ui.available_width();
    ui.painter().hline(bar_top.x..=(bar_top.x + width), bar_top.y, Stroke::new(1.0, t.border));
    ui.add_space(10.0);

    let items = [
        ("Strata 조사 파일", "광령리1.gpkg · 읽기만 합니다"),
        ("desca 단면", "2개 · A–A′, Tr-1 북벽"),
        ("측점", "기준점 4 · 측점 38"),
        ("밑그림", "스캔 6장 · 정사영상 1장"),
        ("보고서 글꼴", "맑은 고딕 · 컴퓨터에 있는 글꼴"),
    ];

    let col_w = width / 5.0;
    let (bar_rect, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    let cy = bar_rect.center().y;

    for (i, (title, subtitle)) in items.iter().enumerate() {
        let x = bar_rect.left() + col_w * i as f32;

        if i > 0 {
            ui.painter().vline(x, (cy - 12.0)..=(cy + 12.0), Stroke::new(1.0, t.border));
        }

        let tx = if i == 0 { x } else { x + 12.0 };
        text(ui, pos2(tx, cy - 7.0), Align2::LEFT_CENTER, *title, bold(12.0), t.text);
        text(ui, pos2(tx, cy + 8.0), Align2::LEFT_CENTER, *subtitle, font(11.0), t.text_dim);
    }
}

/// Draws an archeological template preview vector icon.
fn draw_template_glyph(ui: &Ui, r: Rect, key: &str, t: &Tokens) {
    let c = r.center();
    let ink_stroke = Stroke::new(1.0, t.text);
    let faint_stroke = Stroke::new(0.8, t.border);

    match key {
        "plan" => {
            // Plan preview: feature polygon and reference axes
            let poly = Rect::from_center_size(c - vec2(6.0, 4.0), vec2(28.0, 16.0));
            ui.painter().rect_stroke(poly, CornerRadius::same(3), ink_stroke, StrokeKind::Inside);
            ui.painter().hline((r.left() + 4.0)..=(r.right() - 4.0), c.y + 6.0, faint_stroke);
        }
        "section" => {
            // Section preview: horizontal elevation lines with profile
            for dy in [-12.0, -4.0, 4.0, 12.0] {
                ui.painter().hline((r.left() + 4.0)..=(r.right() - 4.0), c.y + dy, faint_stroke);
            }
            let pts = vec![
                pos2(r.left() + 6.0, c.y - 8.0),
                pos2(c.x - 8.0, c.y + 2.0),
                pos2(c.x + 8.0, c.y + 10.0),
                pos2(r.right() - 6.0, c.y - 4.0),
            ];
            ui.painter().add(egui::Shape::line(pts, ink_stroke));
        }
        "finds" => {
            // Finds preview: pottery section profile with center line
            ui.painter().vline(c.x, (r.top() + 4.0)..=(r.bottom() - 4.0), Stroke::new(0.8, t.text));
            let pts_left = vec![
                pos2(c.x - 14.0, r.top() + 8.0),
                pos2(c.x - 18.0, c.y),
                pos2(c.x - 10.0, r.bottom() - 8.0),
                pos2(c.x, r.bottom() - 8.0),
            ];
            ui.painter().add(egui::Shape::line(pts_left, ink_stroke));
            let pts_right = vec![
                pos2(c.x + 14.0, r.top() + 8.0),
                pos2(c.x + 18.0, c.y),
                pos2(c.x + 10.0, r.bottom() - 8.0),
                pos2(c.x, r.bottom() - 8.0),
            ];
            ui.painter().add(egui::Shape::line(pts_right, ink_stroke));
        }
        "siteplan" => {
            // Site layout preview: survey boundary with features
            let pts = vec![
                pos2(r.left() + 6.0, r.top() + 6.0),
                pos2(r.right() - 8.0, r.top() + 10.0),
                pos2(r.right() - 4.0, r.bottom() - 6.0),
                pos2(r.left() + 8.0, r.bottom() - 4.0),
            ];
            ui.painter().add(egui::Shape::closed_line(pts, Stroke::new(0.8, t.text)));
            ui.painter().rect_filled(Rect::from_center_size(c - vec2(8.0, 2.0), vec2(8.0, 6.0)), 1.0, t.border);
            ui.painter().rect_filled(Rect::from_center_size(c + vec2(10.0, 4.0), vec2(10.0, 6.0)), 1.0, t.border);
        }
        "plate" => {
            // Report plate layout preview: document page with figures
            let page = Rect::from_center_size(c, vec2(28.0, 38.0));
            ui.painter().rect_stroke(page, CornerRadius::same(1), ink_stroke, StrokeKind::Inside);
            ui.painter().rect_filled(Rect::from_min_size(page.min + vec2(3.0, 3.0), vec2(22.0, 14.0)), 0.0, t.border);
            ui.painter().hline((page.left() + 3.0)..=(page.right() - 3.0), page.bottom() - 8.0, faint_stroke);
            ui.painter().hline((page.left() + 3.0)..=(page.left() + 14.0), page.bottom() - 4.0, faint_stroke);
        }
        _ => {}
    }
}

/// Draws the Trowel badge logo.
fn draw_trowel_logo(ui: &Ui, r: Rect, t: &Tokens) {
    ui.painter().rect_filled(r, CornerRadius::same(10), t.text);

    let clay = t.clay;
    let pts = vec![
        pos2(r.left() + 10.0, r.bottom() - 11.0),
        pos2(r.center().x, r.top() + 10.0),
        pos2(r.center().x + 6.0, r.top() + 10.0),
        pos2(r.right() - 9.0, r.top() + 17.0),
    ];
    ui.painter().add(egui::Shape::line(pts, Stroke::new(2.5, t.icon_on)));
    ui.painter().hline(
        (r.left() + 10.0)..=(r.right() - 9.0),
        r.bottom() - 11.0,
        Stroke::new(2.5, clay),
    );
}

/// A styled action button (standard or clay).
fn action_button(
    ui: &mut Ui,
    id: egui::Id,
    label: &str,
    is_clay: bool,
    t: &Tokens,
) -> egui::Response {
    let pad = 16.0;
    let font_id = if is_clay { bold(13.0) } else { font(13.0) };
    let text_w = width_of(ui, label, font_id.clone());
    let (rect, _) = ui.allocate_exact_size(vec2(text_w + pad * 2.0, 38.0), Sense::hover());
    let resp = ui.interact(rect, id, Sense::click());

    let (bg_color, text_color) = if is_clay {
        (t.clay, t.icon_on)
    } else {
        (t.input, t.text)
    };

    let fill = if resp.hovered() {
        if is_clay {
            t.action
        } else {
            t.hover
        }
    } else {
        bg_color
    };

    ui.painter().rect(rect, CornerRadius::same(8), fill, Stroke::new(1.0, t.input_border), StrokeKind::Inside);
    text(ui, rect.center(), Align2::CENTER_CENTER, label, font_id, text_color);

    resp
}
