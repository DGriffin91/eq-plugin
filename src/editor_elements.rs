use __core::f64::consts::TAU;
use biquad::{Coefficients, ToHertz, Type};
use imgui::*;
use num_complex::Complex;

use crate::{
    eq::FilterKind,
    units::{butterworth_cascade_q, Units},
};

pub fn map_to_freq(n: f32) -> f32 {
    //0-1 to freq
    //n.powf(4.0).to_range(20.0, 20000.0)
    let n = ((1000.0f32).powf(n) - 1.0) / (1000.0f32 - 1.0);
    n.to_range(20.0, 20000.0)
}

pub fn reverse_map_to_freq(n: f32) -> f32 {
    //(n / 20.0).powf(0.1) - 1.0
    //(n.from_range(20.0, 20000.0)).powf(0.25);
    let n = n.from_range(20.0, 20000.0);
    ((1000.0f32 - 1.0) * n + 1.0).ln() / 1000.0f32.ln()
}

fn draw_hz_line(ui: &Ui, freq: f32, graph_width: f32, graph_height: f32) {
    let [cx, cy] = ui.cursor_screen_pos();
    let x = cx + reverse_map_to_freq(freq) * graph_width;
    ui.get_window_draw_list()
        .add_line([x, cy], [x, cy - graph_height], [1.0, 1.0, 1.0, 0.1])
        .thickness(1.0)
        .build();

    ui.get_window_draw_list().add_text(
        [x, cy - graph_height],
        ui.style_color(StyleColor::Text),
        &ImString::new(format!("{}hz", freq as i32)),
    );
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
        0, 10, 20, 30, 50, 100, 200, 300, 500, 1000, 2000, 3000, 5000, 10000, 20000,
    ]
    .iter()
    {
        draw_hz_line(ui, *n as f32, size[0], size[1]);
    }

    for db in [-12.0, -6.0, 0.0, 6.0, 12.0].iter() {
        draw_db_line(ui, *db, size[0], size[1], db_px_step);
    }
}

pub fn coeffs_from_filter(
    kind: FilterKind,
    freq: f64,
    slope: f64,
    bw_value: f64,
    gain: f64,
    sample_rate: f64,
) -> Vec<Coefficients<f64>> {
    let u_slope = slope as u32;
    let slope_gain = ((slope * 0.5) as u32) as f64;
    let partial_gain = gain / slope_gain;
    let q_offset = bw_value.bw_to_q() * biquad::Q_BUTTERWORTH_F64;

    match kind {
        FilterKind::Bell => {
            vec![Coefficients::<f64>::from_params(
                Type::PeakingEQ(gain),
                sample_rate.hz(),
                freq.hz(),
                bw_value.bw_to_q(),
            )
            .unwrap()]
        }
        FilterKind::LowPass => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::LowPass,
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                )
            }
            coeffs_a
        }
        FilterKind::HighPass => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::HighPass,
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                )
            }
            coeffs_a
        }
        FilterKind::LowShelf => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::LowShelf(partial_gain),
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                )
            }
            coeffs_a
        }
        FilterKind::HighShelf => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::HighShelf(partial_gain),
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                )
            }
            coeffs_a
        }
        FilterKind::Notch => {
            vec![Coefficients::<f64>::from_params(
                Type::Notch,
                sample_rate.hz(),
                freq.hz(),
                (bw_value).bw_to_q(),
            )
            .unwrap()]
        }
        FilterKind::BandPass => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::HighPass,
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                );
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::LowPass,
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                );
            }
            coeffs_a
        }
        FilterKind::Tilt => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::HighShelf(partial_gain * 2.0),
                        sample_rate.hz(),
                        freq.hz(),
                        q_value * q_offset,
                    )
                    .unwrap(),
                )
            }
            coeffs_a
        }
        FilterKind::Mesa => {
            let mut coeffs_a = Vec::new();
            for i in 0..(slope * 0.5) as usize {
                let q_value = butterworth_cascade_q(u_slope, i as u32);
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::LowShelf(-partial_gain),
                        sample_rate.hz(),
                        (freq / (bw_value + 0.5)).min(20000.0).max(20.0).hz(),
                        q_value,
                    )
                    .unwrap(),
                );
                coeffs_a.push(
                    Coefficients::<f64>::from_params(
                        Type::HighShelf(-partial_gain),
                        sample_rate.hz(),
                        (freq * (bw_value + 0.5)).min(20000.0).max(20.0).hz(),
                        q_value,
                    )
                    .unwrap(),
                );
            }
            coeffs_a
        }
        FilterKind::AllPass => {
            vec![Coefficients::<f64>::from_params(
                Type::AllPass,
                sample_rate.hz(),
                freq.hz(),
                (bw_value).bw_to_q(),
            )
            .unwrap()]
        }
    }
}

pub fn first_order_biquad_bode(f_hz: f64, coeffs: &Coefficients<f64>, sample_rate: f64) -> f64 {
    let imag = Complex::new(0.0, 1.0);

    let jw = (-TAU * f_hz * imag / sample_rate).exp();

    let jw_sq = jw * jw;

    let numerator = coeffs.b0 + (coeffs.b1 * jw) + (coeffs.b2 * jw_sq);
    let denominator = (coeffs.a1 * jw) + (coeffs.a2 * jw_sq) + 1.0;
    (numerator / denominator).norm()
}
