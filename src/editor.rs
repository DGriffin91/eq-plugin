use crate::editor_elements::coeffs_from_filter;
use imgui::*;
use imgui_knobs::*;

use crate::{atomic_f64::AtomicF64, editor_elements::*, eq::FilterKind};

use crate::units::{map_to_freq, Units};
use imgui_baseview::{HiDpiMode, ImguiWindow, RenderSettings, Settings};

use crate::eq_effect_parameters::EQEffectParameters;
use crate::parameter::Parameter;

use vst::editor::Editor;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::Arc;

const WINDOW_WIDTH: usize = 1300;
const WINDOW_HEIGHT: usize = 1300;
const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
//const BG_COLOR: [f32; 4] = [0.21 * 1.4, 0.11 * 1.7, 0.25 * 1.4, 1.0];
//const BG_COLOR_TRANSP: [f32; 4] = [0.21 * 1.4, 0.11 * 1.7, 0.25 * 1.4, 0.0];
//const GREEN: [f32; 4] = [0.23, 0.68, 0.23, 1.0];
//const RED: [f32; 4] = [0.98, 0.02, 0.22, 1.0];
const ORANGE: [f32; 4] = [1.0, 0.58, 0.0, 1.0];
const ORANGE_HOVERED: [f32; 4] = [1.0, 0.68, 0.1, 1.0];
//const WAVEFORM_LINES: [f32; 4] = [1.0, 1.0, 1.0, 0.2];
//const TEXT: [f32; 4] = [1.0, 1.0, 1.0, 0.75];
//const DB_LINES: [f32; 4] = [1.0, 1.0, 1.0, 0.15];

pub fn draw_knob(knob: &Knob, wiper_color: &ColorSet, track_color: &ColorSet) {
    knob.draw_arc(
        0.8,
        0.20,
        knob.angle_min,
        knob.angle_max,
        track_color,
        16,
        2,
    );
    if knob.t > 0.01 {
        knob.draw_arc(0.8, 0.21, knob.angle_min, knob.angle, wiper_color, 16, 2);
    }
}

pub fn make_knob(
    ui: &Ui,
    parameter: &Parameter,
    wiper_color: &ColorSet,
    track_color: &ColorSet,
    title_fix: f32,
    n: usize,
) {
    let width = ui.text_line_height() * 4.75;
    let w = ui.push_item_width(width);
    let title = parameter.get_name();
    let knob_id = &ImString::new(format!("##{}_{}_KNOB_CONTORL_", title, n));
    knob_title(ui, &ImString::new(title.clone().to_uppercase()), width);
    let cursor = ui.cursor_pos();
    ui.set_cursor_pos([cursor[0], cursor[1] + 5.0]);
    let mut val = parameter.get_normalized() as f32;
    let knob = Knob::new(
        ui,
        knob_id,
        &mut val,
        0.0,
        1.0,
        parameter.get_normalized_default() as f32,
        width * 0.5,
        true,
    );
    let cursor = ui.cursor_pos();
    ui.set_cursor_pos([cursor[0] + title_fix, cursor[1] - 10.0]);
    knob_title(ui, &ImString::new(parameter.get_display()), width);

    if knob.value_changed {
        //parameter.set(*knob.p_value)
        parameter.set_normalized(*knob.p_value as f64)
    }

    w.pop(ui);
    draw_knob(&knob, wiper_color, track_color);
}

pub struct EditorState {
    pub params: Arc<EQEffectParameters>,
    pub sample_rate: Arc<AtomicF64>,
}

pub struct EQPluginEditor {
    pub is_open: bool,
    pub state: Arc<EditorState>,
}

fn move_cursor(ui: &Ui, x: f32, y: f32) {
    let cursor = ui.cursor_pos();
    ui.set_cursor_pos([cursor[0] + x, cursor[1] + y])
}

fn floating_text(ui: &Ui, text: &str) {
    ui.get_window_draw_list()
        .add_text(ui.cursor_pos(), ui.style_color(StyleColor::Text), text)
}

impl Editor for EQPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        //::log::info!("self.running {}", self.running);
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let settings = Settings {
            window: WindowOpenOptions {
                title: String::from("imgui-baseview demo window"),
                size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            clear_color: (0.0, 0.0, 0.0),
            hidpi_mode: HiDpiMode::Default,
            render_settings: RenderSettings::default(),
        };

        ImguiWindow::open_parented(
            &VstParent(parent),
            settings,
            self.state.clone(),
            |ctx: &mut Context, _state: &mut Arc<EditorState>| {
                ctx.fonts().add_font(&[FontSource::TtfData {
                    data: include_bytes!("../FiraCode-Regular.ttf"),
                    size_pixels: 20.0,
                    config: None,
                }]);
            },
            |_run: &mut bool, ui: &Ui, state: &mut Arc<EditorState>| {
                let w = Window::new(im_str!("Example 1: Basic sliders"))
                    .size([WINDOW_WIDTH_F, WINDOW_HEIGHT_F], Condition::Appearing)
                    .position([0.0, 0.0], Condition::Appearing)
                    .draw_background(false)
                    .no_decoration()
                    .movable(false);
                w.build(&ui, || {
                    let graph_width = 1200.0;
                    let graph_height = 500.0;

                    let db_px_step = graph_height / 40.0;

                    let [cx, cy] = ui.cursor_screen_pos();
                    let [mx, my] = ui.io().mouse_pos;
                    let [px, py] = [
                        map_to_freq((mx - cx) / graph_width),
                        -(my - cy - (graph_height * 0.5)) / 10.0,
                    ];
                    let [px, py] = [px.min(20000.0).max(10.0), py.min(24.0).max(-96.0)];

                    ui.get_window_draw_list().add_text(
                        [mx - 40.0, my - 25.0],
                        ui.style_color(StyleColor::Text),
                        &ImString::new(format!("{}hz {}dB", px as i32, py)),
                    );

                    let sample_rate = state.sample_rate.get();

                    let highlight = ColorSet::new(ORANGE, ORANGE_HOVERED, ORANGE_HOVERED);
                    let lowlight = ColorSet::from(BLACK);
                    let params = &state.params;

                    let mut graph_y_values = vec![0.0f32; graph_width as usize];

                    for band in state.params.bands.iter() {
                        let kind = band.get_kind();
                        let freq = band.freq.get();
                        let slope = band.get_slope();
                        let bw = band.bw.get();
                        let gain = band.gain.get();
                        let coeffs_set =
                            coeffs_from_filter(kind, freq, slope, bw, gain, sample_rate);
                        for (i, graph_y) in graph_y_values.iter_mut().enumerate() {
                            let f_hz = map_to_freq((i as f32) / graph_width) as f64;
                            let mut y = 1.0f64;
                            for coeffs in coeffs_set.iter() {
                                y *= first_order_biquad_bode(f_hz, coeffs, sample_rate);
                                if kind == FilterKind::Mesa {
                                    let gain = ((gain / (slope * 2.0)) * 2.0).db_to_lin();
                                    y *= gain;
                                } else if kind == FilterKind::Tilt {
                                    let gain = ((gain / (slope * 0.5)) * -1.0).db_to_lin();
                                    y *= gain;
                                }
                            }
                            *graph_y += -(y.lin_to_db()) as f32;
                        }
                    }

                    //ui.text(&ImString::new(format!("{}", graph_y_values[100])));
                    draw_eq_graph(
                        ui,
                        im_str!("test"),
                        [graph_width, graph_height],
                        db_px_step,
                        2.0,
                        graph_width as usize,
                        |i| graph_y_values[i],
                    );

                    ui.columns(6, im_str!("cols"), false);
                    for (i, band) in params.bands.iter().enumerate() {
                        ui.text(&ImString::new(band.get_kind().to_string()));
                        make_knob(ui, &band.kind, &highlight, &lowlight, 0.0, i);
                        ui.next_column();
                        make_knob(ui, &band.freq, &highlight, &lowlight, 0.0, i);
                        ui.next_column();
                        make_knob(ui, &band.gain, &highlight, &lowlight, 0.0, i);
                        ui.next_column();
                        make_knob(ui, &band.bw, &highlight, &lowlight, 0.0, i);
                        ui.next_column();
                        make_knob(ui, &band.slope, &highlight, &lowlight, 0.0, i);
                        ui.next_column();
                        let slope = band.get_slope() as i32;
                        for i in (2..=16).step_by(2) {
                            if ui.radio_button_bool(
                                &ImString::new(format!("{}dB", i * 6)),
                                slope == i,
                            ) {
                                band.slope.set(i as f64);
                                break;
                            }
                        }
                        ui.next_column();
                    }
                });
                //ui.show_demo_window(run);
            },
        );

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}

struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}
