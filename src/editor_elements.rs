use imgui::*;

use crate::units::reverse_map_to_freq;

fn draw_hz_line(ui: &Ui, freq: f32, graph_width: f32, graph_height: f32, draw_text: bool) {
    let [cx, cy] = ui.cursor_screen_pos();
    let x = cx + reverse_map_to_freq(freq) * graph_width;
    ui.get_window_draw_list()
        .add_line([x, cy], [x, cy - graph_height], [1.0, 1.0, 1.0, 0.2])
        .thickness(1.0)
        .build();

    if draw_text {
        ui.get_window_draw_list().add_text(
            [x, cy - graph_height],
            ui.style_color(StyleColor::Text),
            &ImString::new(format!("{}hz", freq as i32)),
        );
    }
}

fn draw_db_line(ui: &Ui, db: f32, graph_width: f32, graph_height: f32, db_px_step: f32) {
    let [cx, cy] = ui.cursor_screen_pos();
    let db_height = cy + (-db) * db_px_step - graph_height / 2.0;
    ui.get_window_draw_list()
        .add_line(
            [cx, db_height],
            [cx + graph_width, db_height],
            [1.0, 1.0, 1.0, 0.1],
        )
        .thickness(1.0)
        .build();
    ui.get_window_draw_list().add_text(
        [cx, db_height],
        ui.style_color(StyleColor::Text),
        &ImString::new(format!("{}db", db)),
    );
}

pub fn draw_eq_graph<F: Fn(usize) -> f32>(
    ui: &Ui,
    id: &ImStr,
    size: [f32; 2],
    db_px_step: f32,
    thinkness: f32,
    length: usize,
    value_fn: F,
) {
    let [cx, mut cy] = ui.cursor_screen_pos();
    cy += 4.0; //TODO off by a bit
    ui.invisible_button(id, size);

    let mut color = if ui.is_item_hovered() {
        ui.style_color(StyleColor::PlotLinesHovered)
    } else {
        ui.style_color(StyleColor::PlotLines)
    };
    let scale = (size[0] as f32 / length as f32) as f32;
    color[3] = (color[3] * 0.9).min(1.0).max(0.0);
    let v_center = size[1] / 2.0;
    let mut last = value_fn(0) * db_px_step;
    {
        let draw_list = ui.get_window_draw_list();
        for i in 0..length {
            let fi = i as f32;
            let next = value_fn(i) * db_px_step;
            let x_ofs = if (next - last).abs() < 1.0 { 1.0 } else { 0.0 };
            let p1 = [cx + fi * scale, cy + v_center + last];
            let p2 = [cx + fi * scale + x_ofs, cy + v_center + next];
            if !(p1[1] < 0.0 || p1[1] > size[1] || p2[1] < 0.0 || p2[1] > size[1]) {
                draw_list
                    .add_line(p1, p2, color)
                    .thickness(thinkness)
                    .build();
            }
            last = next;
        }
    }

    for n in [
        20, 30, 50, 100, 200, 300, 500, 1000, 2000, 3000, 5000, 10000, 20000,
    ]
    .iter()
    {
        draw_hz_line(ui, *n as f32, size[0], size[1], true);
    }

    for n in [
        40, 60, 70, 80, 90, 400, 600, 700, 800, 900, 4000, 6000, 7000, 8000, 9000,
    ]
    .iter()
    {
        draw_hz_line(ui, *n as f32, size[0], size[1], false);
    }

    for db in [-12.0, -6.0, 0.0, 6.0, 12.0].iter() {
        draw_db_line(ui, *db, size[0], size[1], db_px_step);
    }
}
