//! The menu bar: the app's menus with the Drawing menu (design 2.1 §3) before View. Its first two
//! items run (J005): Change Weights at Once… opens its card, Revert to the Legend puts the
//! selected lines back to their row's look, or every line set apart when nothing is selected. The
//! others come with later jobs; until then they show dimmed, each with its job.

use std::cell::Cell;

use serde_json::json;

use super::legend::{Card, open_card, revert};
use super::*;

/// A Drawing menu item that runs.
#[derive(Clone, Copy)]
enum Item {
    Weights,
    Revert,
    InsertLegend,
    PlateLayout,
    StyleTemplate,
    GpkgSurvey,
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(MENU).exact_size(MENU_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let mut bar =
            ui.new_child(egui::UiBuilder::new().max_rect(r.shrink2(vec2(6.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
        let (open, chosen) = (app.session.active().is_some(), Cell::new(None));
        let body = |ui: &mut Ui| drawing_menu(ui, open, &chosen);
        let extra = crate::menus::ExtraMenu { before: "View", title: tw!("Drawing"), body: &body };
        let (_, title) = crate::menus::menu_bar_with(app, &mut bar, 12.0, Some(10.0), Some(&extra));
        // The new menu is underlined across its title, as in the mockup.
        if let Some(tr) = title {
            let under = Rect::from_min_max(pos2(tr.left(), r.center().y + 7.0), pos2(tr.right(), r.center().y + 9.0));
            ui.painter().rect_filled(under, 0.0, t.row_selected);
        }
        match chosen.get() {
            Some(Item::Weights) => open_card(app, ui.ctx(), Card::Weights, None),
            Some(Item::Revert) => {
                let selected = app.session.active().is_some_and(|st| !st.selection.objects.is_empty());
                if let Err(e) = revert(app, if selected { json!({}) } else { json!({ "all": true }) }) {
                    app.status(e);
                }
            }
            Some(Item::InsertLegend) => {
                let _ = app.run("style.generateLegend", json!({ "filterUnused": true, "replaceExisting": true }));
            }
            Some(Item::PlateLayout) => super::plate::toggle_dialog(ui.ctx()),
            Some(Item::StyleTemplate) => super::style_dialog::toggle_dialog(ui.ctx()),
            Some(Item::GpkgSurvey) => super::gpkg_browser::toggle_dialog(ui.ctx()),
            None => {}
        }
    });
}

/// The Drawing menu's items, in groups: the active items that run, then upcoming ones.
fn drawing_menu(ui: &mut Ui, open: bool, chosen: &Cell<Option<Item>>) {
    let live = [
        (tw!("Change Weights at Once…"), tw!("Change every line's weight at once"), Item::Weights),
        (tw!("Revert to the Legend"), tw!("The selected lines, or every line set apart when nothing is selected"), Item::Revert),
        ("캔버스 범례표 삽입", "도면에서 실제 사용된 줄로 자동 범례표를 삽입합니다 (J018)", Item::InsertLegend),
        ("보고서 도판 배치 및 표제란…", "출판용 도판 배치·표제란·축척 막대·벡터 PDF 내보내기 (J012)", Item::PlateLayout),
        ("도면 양식 템플릿 관리 (.trowel-style)…", "기관별 표준 양식 비교 및 병합 (J018)", Item::StyleTemplate),
        ("Strata GeoPackage 발굴조사 데이터…", "현장 측량 데이터 레이어 불러오기 및 동기화 (J019)", Item::GpkgSurvey),
    ];
    for (label, tip, item) in live {
        if ui.add_enabled(open, egui::Button::new(label)).on_hover_text(tip).clicked() {
            chosen.set(Some(item));
            ui.close();
        }
    }
    let groups: [&[(&str, &str)]; 2] = [
        &[(tw!("Cut Face"), "J014"), (tw!("Make Level Lines"), "J015")],
        &[(tw!("Scale and Coordinate System…"), "J006"), (tw!("Align Underlay…"), "J008")],
    ];
    for group in groups {
        ui.separator();
        for (label, job) in group.iter() {
            ui.add_enabled(false, egui::Button::new(*label)).on_disabled_hover_text(soon(label, job));
        }
    }
}
