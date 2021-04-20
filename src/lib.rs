//! EQ baseview imgui plugin

/*
Ideas:
    Vertical spectrogram like BlueCat and fl studio
        also fft spectrum option with peak detection (see mequalizer)
    Dynamic bands (use band pass as trigger)
    Solo selected band with band pass
    Tilt style bands
        Initial done, maybe also do flat tilt
    Handles for moving only Vertical/Horizontal movement?
    Look at compensating for frequency warping
    Interpolate to avoid zipper effects when automating
        DONE - gain/hz/bw
        still getting more noise than proQ try smoothing coeffs
    mid/side/left/right percentages
    text input
    Eventually fir filters?
    Oversampling? - Probably not
    DONE - Raised cosine (2 shelf filters?)
    DONE - Look at svf https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf
*/

#[macro_use]
extern crate vst;

mod editor;
pub mod editor_elements;
pub mod eq;
mod eq_effect_parameters;
mod parameter;
mod svf;
pub mod units;

mod atomic_f64;

use editor::{EQPluginEditor, EditorState};
use eq::FilterbandStereo;
use eq_effect_parameters::{BandParameters, EQEffectParameters};

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use std::sync::Arc;

use atomic_f64::AtomicF64;

const FILTER_COUNT: usize = 4;
const FILTER_POLE_COUNT: usize = 16;

pub struct EditorFilterData {
    pub params: Arc<BandParameters>,
}

struct EQPlugin {
    params: Arc<EQEffectParameters>,
    editor: Option<EQPluginEditor>,
    filter_bands: Vec<FilterbandStereo>,
    time: Arc<AtomicF64>,
    sample_rate: Arc<AtomicF64>,
    block_size: i64,
}

impl Default for EQPlugin {
    fn default() -> Self {
        let params = Arc::new(EQEffectParameters::default());
        let time = Arc::new(AtomicF64::new(0.0));
        let sample_rate = Arc::new(AtomicF64::new(44100.0));

        let filter_bands = (0..FILTER_COUNT)
            .map(|_| FilterbandStereo::new(48000.0))
            .collect::<Vec<FilterbandStereo>>();

        Self {
            params: params.clone(),
            sample_rate: sample_rate.clone(),
            block_size: 128,
            time: time.clone(),
            editor: Some(EQPluginEditor {
                is_open: false,
                state: Arc::new(EditorState {
                    params: params.clone(),
                    sample_rate: sample_rate.clone(),
                }),
            }),
            filter_bands,
        }
    }
}

fn setup_logging() {
    let log_folder = ::dirs::home_dir().unwrap().join("tmp");

    let _ = ::std::fs::create_dir(log_folder.clone());

    let log_file = ::std::fs::File::create(log_folder.join("IMGUIBaseviewEQ.log")).unwrap();

    let log_config = ::simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .build();

    let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);

    ::log_panics::init();

    ::log::info!("init");
}

impl Plugin for EQPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "IMGUI EQ in Rust 0.1".to_string(),
            vendor: "DGriffin".to_string(),
            unique_id: 237953123,
            version: 2,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: self.params.len() as i32,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate.set(rate as f64);
    }

    fn set_block_size(&mut self, block_size: i64) {
        self.block_size = block_size;
    }

    fn init(&mut self) {
        setup_logging();
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        self.time
            .set(self.time.get() + (1.0 / self.sample_rate.get()) * self.block_size as f64);

        let (inputs, outputs) = buffer.split();
        let (inputs_left, inputs_right) = inputs.split_at(1);
        let (mut outputs_left, mut outputs_right) = outputs.split_at_mut(1);

        let inputs_stereo = inputs_left[0].iter().zip(inputs_right[0].iter());
        let outputs_stereo = outputs_left[0].iter_mut().zip(outputs_right[0].iter_mut());

        for (input_pair, output_pair) in inputs_stereo.zip(outputs_stereo) {
            for (i, band) in self.params.bands.iter().enumerate() {
                self.filter_bands[i].update(
                    band.get_kind(),
                    band.freq.get(),
                    band.gain.get(),
                    band.bw.get(),
                    band.get_slope(),
                    self.sample_rate.get(),
                );
            }

            let (input_l, input_r) = input_pair;
            let (output_l, output_r) = output_pair;

            let mut l = *input_l as f64;
            let mut r = *input_r as f64;

            for i in 0..self.filter_bands.len() {
                let [l_n, r_n] = self.filter_bands[i].process(l, r);
                l = l_n;
                r = r_n;
            }

            *output_l = l as f32;
            *output_r = r as f32;
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for EQEffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        if (index as usize) < self.len() {
            self[index as usize].get_normalized() as f32
        } else {
            0.0
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        if (index as usize) < self.len() {
            self[index as usize].set_normalized(val as f64);
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.

    fn get_parameter_text(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_display()
        } else {
            "".to_string()
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_name()
        } else {
            "".to_string()
        }
    }
}

plugin_main!(EQPlugin);
