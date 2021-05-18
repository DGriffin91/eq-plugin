use crate::FILTER_POLE_COUNT;

use super::parameter::Parameter;

use core::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BandType {
    Bell,
    LowPass,
    HighPass,
    LowShelf,
    HighShelf,
    Notch,
    BandPass,
    AllPass,
}

impl BandType {
    pub fn from_u8(value: u8) -> BandType {
        match value {
            1 => BandType::Bell,
            2 => BandType::LowPass,
            3 => BandType::HighPass,
            4 => BandType::LowShelf,
            5 => BandType::HighShelf,
            6 => BandType::Notch,
            7 => BandType::BandPass,
            8 => BandType::AllPass,
            _ => BandType::LowPass,
        }
    }
}

impl fmt::Display for BandType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct BandParameters {
    pub kind: Parameter,
    pub freq: Parameter,
    pub gain: Parameter,
    pub bw: Parameter,
    pub slope: Parameter,
}

impl BandParameters {
    pub fn get_kind(&self) -> BandType {
        return BandType::from_u8(self.kind.get() as u8);
    }

    pub fn get_slope(&self) -> f64 {
        self.slope.get() as u8 as f64
    }

    pub fn dsp_update(&self) -> bool {
        if self.kind.dsp_update() {
            true
        } else if self.freq.dsp_update() {
            true
        } else if self.gain.dsp_update() {
            true
        } else if self.bw.dsp_update() {
            true
        } else if self.slope.dsp_update() {
            true
        } else {
            false
        }
    }
}

pub struct EQEffectParameters {
    pub bands: Vec<Arc<BandParameters>>,
}

use std::{ops::Index, sync::Arc};

impl Index<usize> for EQEffectParameters {
    type Output = Parameter;
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.bands[0].kind,
            1 => &self.bands[0].freq,
            2 => &self.bands[0].gain,
            3 => &self.bands[0].bw,
            4 => &self.bands[0].slope,
            5 => &self.bands[1].kind,
            6 => &self.bands[1].freq,
            7 => &self.bands[1].gain,
            8 => &self.bands[1].bw,
            9 => &self.bands[1].slope,
            10 => &self.bands[2].kind,
            11 => &self.bands[2].freq,
            12 => &self.bands[2].gain,
            13 => &self.bands[2].bw,
            14 => &self.bands[2].slope,
            15 => &self.bands[3].kind,
            16 => &self.bands[3].freq,
            17 => &self.bands[3].gain,
            18 => &self.bands[3].bw,
            19 => &self.bands[3].slope,
            _ => &self.bands[3].kind,
        }
    }
}

impl EQEffectParameters {
    pub fn len(&self) -> usize {
        16
    }
}

fn new_band_pram_set(n: usize) -> BandParameters {
    BandParameters {
        kind: Parameter::new(
            &format!("Band {} Type", n),
            1.0,
            1.0,
            10.0,
            |x| BandType::from_u8(x as u8).to_string(),
            |x| x,
            |x| x,
        ),
        freq: Parameter::new(
            &format!("Band {} hz", n),
            1000.0,
            20.0,
            20000.0,
            |x| format!("hz {:.2}", x),
            |x| x.powf(2.0),
            |x| x.powf(0.5),
        ),
        gain: Parameter::new(
            &format!("Band {} dB", n),
            0.0,
            -24.0,
            24.0,
            |x| format!("dB {:.2}", x),
            |x| x,
            |x| x,
        ),
        bw: Parameter::new(
            &format!("Band {} BW", n),
            1.0,
            0.1,
            24.0,
            |x| format!("BW {:.2}", x),
            |x| x,
            |x| x,
        ),
        slope: Parameter::new(
            &format!("Band {} Slope", n),
            1.0,
            1.0,
            FILTER_POLE_COUNT as f64,
            |x| format!("Slope {:.2}", x),
            |x| x,
            |x| x,
        ),
    }
}

impl Default for EQEffectParameters {
    fn default() -> EQEffectParameters {
        EQEffectParameters {
            bands: (0..4)
                .map(|_| Arc::new(new_band_pram_set(1)))
                .collect::<Vec<Arc<BandParameters>>>(),
        }
    }
}
