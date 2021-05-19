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
    DONE - Interpolate to avoid zipper effects when automating
    mid/side/left/right percentages
    text input
    Eventually fir filters?
    Oversampling? - Probably not
    DONE - Raised cosine (2 shelf filters?)
    DONE - Look at svf https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf
*/

use assert_no_alloc::*;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[macro_use]
extern crate vst;

mod editor;
pub mod editor_elements;
mod eq_effect_parameters;
mod parameter;
pub mod units;

mod atomic_bool;
mod atomic_f64;

use audio_filters::filter_band::{FilterBand, FilterBandCoefficients};

use editor::{EQPluginEditor, EditorState};
use eq_effect_parameters::{BandParameters, BandType, EQEffectParameters};

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use std::sync::Arc;

use atomic_f64::AtomicF64;

const FILTER_COUNT: usize = 4;
const FILTER_POLE_COUNT: usize = 16;

fn get_coefficients<T: audio_filters::units::FP>(
    kind: BandType,
    f0: T,
    gain: T,
    bw: T,
    slope: T,
    fs: T,
) -> FilterBandCoefficients<T> {
    match kind {
        BandType::Bell => FilterBandCoefficients::bell(f0, gain, bw, fs),
        BandType::LowPass => FilterBandCoefficients::lowpass(f0, bw, slope, fs),
        BandType::HighPass => FilterBandCoefficients::highpass(f0, bw, slope, fs),
        BandType::LowShelf => FilterBandCoefficients::lowshelf(f0, gain, bw, slope, fs),
        BandType::HighShelf => FilterBandCoefficients::highshelf(f0, gain, bw, slope, fs),
        BandType::Notch => FilterBandCoefficients::notch(f0, gain, bw, fs),
        BandType::BandPass => FilterBandCoefficients::bandpass(f0, gain, bw, fs),
        BandType::AllPass => FilterBandCoefficients::allpass(f0, bw, slope, fs),
    }
}

pub struct EditorFilterData {
    pub params: Arc<BandParameters>,
}

struct EQPlugin {
    params: Arc<EQEffectParameters>,
    editor: Option<EQPluginEditor>,
    filter_bands_left: Vec<FilterBand<f64>>,
    filter_bands_right: Vec<FilterBand<f64>>,
    time: Arc<AtomicF64>,
    sample_rate: Arc<AtomicF64>,
    block_size: i64,
}

impl Default for EQPlugin {
    fn default() -> Self {
        let params = Arc::new(EQEffectParameters::default());
        let time = Arc::new(AtomicF64::new(0.0));
        let sample_rate = Arc::new(AtomicF64::new(48000.0));

        let coeffs = FilterBandCoefficients::bell(1000.0, 0.0, 1.0, 48000.0);

        let filter_bands_left = (0..FILTER_COUNT)
            .map(|_| FilterBand::from(&coeffs))
            .collect::<Vec<FilterBand<f64>>>();

        let filter_bands_right = (0..FILTER_COUNT)
            .map(|_| FilterBand::from(&coeffs))
            .collect::<Vec<FilterBand<f64>>>();

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
            filter_bands_left,
            filter_bands_right,
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

    let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::max(), log_config, log_file);

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
        //setup_logger();
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        //let b: i32 = a.iter().sum();
        //println!("{}", b);
        assert_no_alloc(|| {
            println!("{}", vec![1.0][0]);
            self.time
                .set(self.time.get() + (1.0 / self.sample_rate.get()) * self.block_size as f64);
            let sample_rate = self.sample_rate.get();

            let (inputs, outputs) = buffer.split();
            let (inputs_left, inputs_right) = inputs.split_at(1);
            let (mut outputs_left, mut outputs_right) = outputs.split_at_mut(1);

            let inputs_stereo = inputs_left[0].iter().zip(inputs_right[0].iter());
            let outputs_stereo = outputs_left[0].iter_mut().zip(outputs_right[0].iter_mut());

            for (input_pair, output_pair) in inputs_stereo.zip(outputs_stereo) {
                for (i, band) in self.params.bands.iter().enumerate() {
                    if !band.dsp_update() {
                        continue;
                    }
                    let f0 = band.freq.get() as f64;
                    let gain = band.gain.get() as f64;
                    let bw = band.bw.get() as f64;
                    let slope = band.get_slope() as f64;
                    let fs = sample_rate as f64;

                    let coeffs = get_coefficients(band.get_kind(), f0, gain, bw, slope, fs);

                    self.filter_bands_left[i].update(&coeffs);
                    self.filter_bands_right[i].update(&coeffs);
                }

                let (input_l, input_r) = input_pair;
                let (output_l, output_r) = output_pair;

                let mut l = *input_l as f64;
                let mut r = *input_r as f64;

                for i in 0..self.filter_bands_left.len() {
                    l = (self.filter_bands_left[i].process)(&mut self.filter_bands_left[i], l);
                    r = (self.filter_bands_right[i].process)(&mut self.filter_bands_right[i], r);
                }

                *output_l = l as f32;
                *output_r = r as f32;
            }
        });
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
