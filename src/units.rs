use ringbuf::Consumer;

const LN_2_F32: f32 = 0.6931471805599453; //(2.0f32).ln()
const LN_2_F64: f64 = 0.6931471805599453; //(2.0f32).ln()

/// Used to implement conversions to the Hertz struct
pub trait Units<T> {
    /// From hertz
    fn to_range(self, bottom: T, top: T) -> T;
    fn from_range(self, bottom: T, top: T) -> T;
    fn db_to_lin(self) -> T;
    fn lin_to_db(self) -> T;
    fn sign(self, b: T) -> T;
    fn bw_to_q(self, f0: T, fs: T) -> T;
}

impl Units<f64> for f64 {
    fn to_range(self, bottom: f64, top: f64) -> f64 {
        self * (top - bottom) + bottom
    }
    fn from_range(self, bottom: f64, top: f64) -> f64 {
        (self - bottom) / (top - bottom)
    }
    fn db_to_lin(self) -> f64 {
        (10.0f64).powf(self * 0.05)
    }
    fn lin_to_db(self) -> f64 {
        self.max(0.0).log(10.0) * 20.0
    }
    fn sign(self, b: f64) -> f64 {
        if b < 0.0 {
            -self
        } else {
            self
        }
    }
    fn bw_to_q(self, _f0: f64, _fs: f64) -> f64 {
        // Tried to compensate for q squashing at high frequencies
        // but seems too extreme at the very top
        //let w0 = 2.0 * PI * f0 / fs;
        //1.0 / (2.0 * (LN_2_F64 / 2.0 * self * w0 / (w0).sin()).sinh())

        1.0 / (2.0 * (LN_2_F64 / 2.0 * self).sinh())
    }
}

impl Units<f32> for f32 {
    //Just a copy of the f64 version with units swapped
    fn to_range(self, bottom: f32, top: f32) -> f32 {
        self * (top - bottom) + bottom
    }
    fn from_range(self, bottom: f32, top: f32) -> f32 {
        (self - bottom) / (top - bottom)
    }
    fn db_to_lin(self) -> f32 {
        (10.0f32).powf(self * 0.05)
    }
    fn lin_to_db(self) -> f32 {
        self.max(0.0).log(10.0) * 20.0
    }
    fn sign(self, b: f32) -> f32 {
        if b < 0.0 {
            -self
        } else {
            self
        }
    }
    fn bw_to_q(self, _f0: f32, _fs: f32) -> f32 {
        // Tried to compensate for q squashing at high frequencies
        // but seems too extreme at the very top
        //let w0 = 2.0 * std::f32::consts::PI * f0 / fs;
        //1.0 / (2.0 * (LN_2_F32 / 2.0 * self * w0 / (w0).sin()).sinh())

        1.0 / (2.0 * (LN_2_F32 / 2.0 * self).sinh())
    }
}

pub fn map_to_freq(n: f32) -> f32 {
    //0-1 to freq
    let n = ((1000.0f32).powf(n) - 1.0) / (1000.0f32 - 1.0);
    n.to_range(20.0, 20000.0)
}

pub fn reverse_map_to_freq(n: f32) -> f32 {
    let n = n.from_range(20.0, 20000.0);
    ((1000.0f32 - 1.0) * n + 1.0).ln() / 1000.0f32.ln()
}

pub struct VariableRingBuffer {
    buffer: Vec<f32>,
    position: usize,
    size: usize,
}

impl VariableRingBuffer {
    pub fn new(init_size: usize, max_size: usize) -> VariableRingBuffer {
        VariableRingBuffer {
            buffer: vec![0.0; max_size],
            position: 0,
            size: init_size,
        }
    }

    pub fn push(&mut self, value: f32) {
        self.buffer[self.position] = value;
        self.position = (self.position + 1) % self.size;
    }

    pub fn oldest(&self) -> f32 {
        self.buffer[self.position]
    }

    pub fn get(&self, index: usize) -> f32 {
        let pos = self.position + index;
        if pos > self.size - 1 {
            self.buffer[pos - self.size]
        } else {
            self.buffer[pos]
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn resize(&mut self, new_size: usize) {
        self.size = new_size.min(self.buffer.len());
        self.position = 0;
        for i in self.buffer.iter_mut() {
            *i = 0.0;
        }
    }
}
pub struct AccumulatingRMS {
    buffer: VariableRingBuffer,
    rms: f32,
}

impl AccumulatingRMS {
    pub fn new(sample_rate: usize, rms_size_ms: f32, rms_max_size_samp: usize) -> AccumulatingRMS {
        AccumulatingRMS {
            buffer: VariableRingBuffer::new(
                ((sample_rate as f32) * (rms_size_ms / 1000.0)) as usize,
                rms_max_size_samp,
            ),
            rms: 0.0,
        }
    }
    pub fn resize(&mut self, sample_rate: usize, rms_size_ms: f32) {
        let new_size = (((sample_rate as f32) * (rms_size_ms / 1000.0)) as usize).max(1);
        if new_size != self.buffer.size() {
            self.buffer.resize(new_size);
            self.rms = 0.0;
        }
    }
    pub fn process(&mut self, value: f32) -> f32 {
        let new_rms_sample = value.powi(2);

        //remove the oldest rms value, add new one
        self.rms += -self.buffer.oldest() + new_rms_sample;
        self.buffer.push(new_rms_sample);
        (self.rms / self.buffer.size() as f32).sqrt()
    }
}

//find a better name?
pub struct ConsumerDump<T> {
    pub data: Vec<T>,
    consumer: Consumer<T>,
    max_size: usize,
}

impl<T> ConsumerDump<T> {
    pub fn new(consumer: Consumer<T>, max_size: usize) -> ConsumerDump<T> {
        ConsumerDump {
            data: Vec::new(),
            consumer,
            max_size,
        }
    }

    pub fn consume(&mut self) {
        for _ in 0..self.consumer.len() {
            if let Some(n) = self.consumer.pop() {
                self.data.push(n);
            } else {
                break;
            }
        }
        self.trim_data()
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        self.trim_data();
    }

    pub fn trim_data(&mut self) {
        //Trims from the start of the vec
        let data_len = self.data.len();
        if data_len > self.max_size {
            self.data.drain(0..(data_len - self.max_size).max(0));
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Smooth {
    pub target: f64,
    pub n: f64,
    pub attack: f64,
}

impl Smooth {
    pub fn new(n: f64) -> Smooth {
        Smooth {
            target: n,
            n,
            attack: 0.1,
        }
    }

    pub fn step(&mut self, sample_rate: f64) {
        let factor = 1.0 / (sample_rate * self.attack);
        self.n += factor * (self.target - self.n);
    }
}
