use crate::FILTER_POLE_COUNT;

use super::parameter::Parameter;

use core::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BandKind {
    Bell,
    LowPass,
    HighPass,
    LowShelf,
    HighShelf,
    Notch,
    BandPass,
    AllPass,
}

impl BandKind {
    pub fn from_u8(value: u8) -> BandKind {
        match value {
            0 => BandKind::Bell,
            1 => BandKind::LowPass,
            2 => BandKind::HighPass,
            3 => BandKind::LowShelf,
            4 => BandKind::HighShelf,
            5 => BandKind::Notch,
            6 => BandKind::BandPass,
            7 => BandKind::AllPass,
            _ => BandKind::LowPass,
        }
    }
}

impl fmt::Display for BandKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BandMode {
    Butterworth,
    LinkwitzRiley,
}

impl BandMode {
    pub fn from_u8(value: u8) -> BandMode {
        match value {
            0 => BandMode::Butterworth,
            1 => BandMode::LinkwitzRiley,
            _ => BandMode::Butterworth,
        }
    }
}

impl fmt::Display for BandMode {
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
    pub mode: Parameter,
}

impl BandParameters {
    pub fn get_kind(&self) -> BandKind {
        return BandKind::from_u8(self.kind.get() as u8);
    }

    pub fn get_slope(&self) -> f64 {
        self.slope.get().floor()
    }

    pub fn get_mode(&self) -> f64 {
        self.mode.get().floor()
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
            5 => &self.bands[0].mode,
            6 => &self.bands[1].kind,
            7 => &self.bands[1].freq,
            8 => &self.bands[1].gain,
            9 => &self.bands[1].bw,
            10 => &self.bands[1].slope,
            11 => &self.bands[1].mode,
            12 => &self.bands[2].kind,
            13 => &self.bands[2].freq,
            14 => &self.bands[2].gain,
            15 => &self.bands[2].bw,
            16 => &self.bands[2].slope,
            17 => &self.bands[2].mode,
            18 => &self.bands[3].kind,
            19 => &self.bands[3].freq,
            20 => &self.bands[3].gain,
            21 => &self.bands[3].bw,
            22 => &self.bands[3].slope,
            23 => &self.bands[3].mode,
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
            0.0,
            0.0,
            10.0,
            |x| BandKind::from_u8(x as u8).to_string(),
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
        mode: Parameter::new(
            &format!("Band {} mode", n),
            0.0,
            0.0,
            2.0,
            |x| BandMode::from_u8(x as u8).to_string(),
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
