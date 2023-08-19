use rodio::{OutputStream, Sink, Source};

use std::sync::{Arc, Mutex};
use std::time::Duration;

const SAMPLE_RATE: u32 = 65536;

const SAMPLE_AVERAGING: usize = 1; //20;

const SQUARE_WAVE_PATTERN_DUTY_0: [u8; 32] = [
    0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf,
    0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0, 0, 0, 0,
];

const SQUARE_WAVE_PATTERN_DUTY_1: [u8; 32] = [
    0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf,
    0xf, 0xf, 0xf, 0xf, 0xf, 0, 0, 0, 0, 0, 0, 0, 0,
];

const SQUARE_WAVE_PATTERN_DUTY_2: [u8; 32] = [
    0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const SQUARE_WAVE_PATTERN_DUTY_3: [u8; 32] = [
    0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0,
];

const SQUARE_WAVE_PATTERNS: [[u8; 32]; 4] = [
    SQUARE_WAVE_PATTERN_DUTY_0,
    SQUARE_WAVE_PATTERN_DUTY_1,
    SQUARE_WAVE_PATTERN_DUTY_2,
    SQUARE_WAVE_PATTERN_DUTY_3,
];

#[derive(Clone, Debug)]
pub struct Wave {
    period_value: u16,
    num_sample: usize,
    wave_pattern: [u8; 32],
    length_timer: u8,
    length_timer_enabled: bool,

    env_initial_volume: f32,
    env_direction: f32,
    env_sweep_pace: u8,

    period_sweep_pace: u8,
    period_sweep_direction: u8,
    period_sweep_slope: u8,
}

impl Wave {
    pub fn new(
        period_value: u16,
        wave_pattern: [u8; 32],
        env_initial_volume: u8,
        env_direction: u8,
        env_sweep_pace: u8,
        length_timer: u8,
        length_timer_enabled: bool,
        period_sweep_pace: u8,
        period_sweep_direction: u8,
        period_sweep_slope: u8,
    ) -> Wave {
        Wave {
            period_value,
            num_sample: 0,
            wave_pattern,
            env_initial_volume: env_initial_volume as f32,
            env_direction: if env_direction == 0 { -1. } else { 1. },
            env_sweep_pace,
            length_timer,
            length_timer_enabled,
            period_sweep_pace,
            period_sweep_direction,
            period_sweep_slope,
        }
    }
}

impl Iterator for Wave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);

        let mut period_value = self.period_value;

        if period_value == 0 {
            return None;
        }

        if self.length_timer_enabled
            && self.length_timer < 64
            && SAMPLE_RATE * (64 - self.length_timer as u32) / 256 < self.num_sample as u32
        {
            return None;
        }

        if self.period_sweep_slope != 0 && self.period_sweep_pace != 0 {
            let sweep_i = ((self.num_sample as f32 * (32768 as f32 / SAMPLE_RATE as f32)) as u32
                / 256)
                / self.period_sweep_pace as u32;

            if self.period_sweep_direction == 0 {
                period_value = 2048
                    - ((2048 - period_value) as f32
                        * f32::powf(
                            f32::powf(2., -(self.period_sweep_slope as f32)) + 1.,
                            sweep_i as f32,
                        )) as u16;
            } else {
                period_value = 2048
                    - ((2048 - period_value) as f32
                        * f32::powf(
                            -f32::powf(2., -(self.period_sweep_slope as f32)) + 1.,
                            sweep_i as f32,
                        )) as u16;
            }

            if period_value > 2048 {
                return None;
            }
        }

        let envelope_time = if self.env_sweep_pace != 0 {
            (self.num_sample as f32 / SAMPLE_RATE as f32) * 64. / self.env_sweep_pace as f32
        } else {
            0.
        };

        let envelope = self.env_initial_volume + (self.env_direction * envelope_time);

        let envelope_boundaries = if envelope > 16. {
            16.
        } else if envelope < 0. {
            0.
        } else {
            envelope
        };

        let mut avg = 0.;

        for n in 0..SAMPLE_AVERAGING {
            if self.num_sample as i32 + n as i32 - SAMPLE_AVERAGING as i32 >= 0 {
                avg += (self.wave_pattern[(((8. * 32768. / (SAMPLE_RATE as f32)
                    * (self.num_sample + n - (SAMPLE_AVERAGING / 2)) as f32
                    / period_value as f32)
                    * 16.)
                    % 32.) as u8 as usize] as f32
                    * 2.
                    - 16.)
                    / 16.;
            }
        }

        Some((avg / SAMPLE_AVERAGING as f32) * envelope_boundaries / 64.)
    }
}

#[derive(Clone, Debug)]
struct MutableWave {
    wave_ch1: Arc<Mutex<Option<Wave>>>,
    wave_ch2: Arc<Mutex<Option<Wave>>>,
    wave_ch3: Arc<Mutex<Option<Wave>>>,
}

impl MutableWave {
    pub fn new(
        wave_ch1: Arc<Mutex<Option<Wave>>>,
        wave_ch2: Arc<Mutex<Option<Wave>>>,
        wave_ch3: Arc<Mutex<Option<Wave>>>,
    ) -> Self {
        Self {
            wave_ch1,
            wave_ch2,
            wave_ch3,
        }
    }
}

impl Iterator for MutableWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let mut res = 0.;

        if let Ok(mut wave_o) = self.wave_ch1.lock() {
            if let Some(wave) = wave_o.as_mut() {
                if let Some(result) = wave.next() {
                    res += result / 4.;
                } else {
                    *wave_o = None;
                }
            }
        }

        if let Ok(mut wave_o) = self.wave_ch2.lock() {
            if let Some(wave) = wave_o.as_mut() {
                if let Some(result) = wave.next() {
                    res += result / 4.;
                } else {
                    *wave_o = None;
                }
            }
        }

        if let Ok(mut wave_o) = self.wave_ch3.lock() {
            if let Some(wave) = wave_o.as_mut() {
                if let Some(result) = wave.next() {
                    res += result / 4.;
                } else {
                    *wave_o = None;
                }
            }
        }

        Some(res)
    }
}

impl Source for MutableWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct AudioSquareChannel {
    wave: Arc<Mutex<Option<Wave>>>,

    pub length_timer: u8,
    pub length_timer_enabled: bool,
    pub on: bool,
    pub period_value: u16,
    pub duty: u8,
    pub initial_volume: u8,
    pub env_direction: u8,
    pub sweep: u8,
    pub period_sweep_pace: u8,
    pub period_sweep_direction: u8,
    pub period_sweep_slope: u8,
}

impl AudioSquareChannel {
    pub fn new(wave: Arc<Mutex<Option<Wave>>>) -> Self {
        Self {
            on: true,
            period_value: 0,
            duty: 0,
            initial_volume: 0,
            env_direction: 0,
            sweep: 0,
            wave,
            length_timer: 0,
            length_timer_enabled: false,
            period_sweep_pace: 0,
            period_sweep_direction: 0,
            period_sweep_slope: 0,
        }
    }

    pub fn update(&mut self) {
        if let Ok(mut wave) = self.wave.lock() {
            if self.on {
                *wave = Some(Wave::new(
                    2048 - self.period_value,
                    SQUARE_WAVE_PATTERNS[self.duty as usize],
                    self.initial_volume,
                    self.env_direction,
                    self.sweep,
                    self.length_timer,
                    self.length_timer_enabled,
                    self.period_sweep_pace,
                    self.period_sweep_direction,
                    self.period_sweep_slope,
                ));
            } else {
                *wave = None;
            }
        }
    }
}

pub struct AudioCustomChannel {
    wave: Arc<Mutex<Option<Wave>>>,

    pub length_timer: u8,
    pub length_timer_enabled: bool,
    pub wave_pattern: [u8; 32],
    pub on: bool,
    pub period_value: u16,
    pub duty: u8,
    pub initial_volume: u8,
    pub env_direction: u8,
    pub sweep: u8,
}

impl AudioCustomChannel {
    pub fn new(wave: Arc<Mutex<Option<Wave>>>) -> Self {
        Self {
            wave_pattern: [0; 32],
            on: true,
            period_value: 0,
            duty: 0,
            initial_volume: 0,
            env_direction: 0,
            sweep: 0,
            wave,
            length_timer: 0,
            length_timer_enabled: false,
        }
    }

    pub fn update(&mut self) {
        if let Ok(mut wave) = self.wave.lock() {
            if self.on {
                *wave = Some(Wave::new(
                    2 * (2048 - (self.period_value * 2)),
                    self.wave_pattern,
                    self.initial_volume,
                    self.env_direction,
                    self.sweep,
                    self.length_timer,
                    self.length_timer_enabled,
                    0,
                    0,
                    0,
                ));
            } else {
                *wave = None;
            }
        }
    }
}

pub struct Audio {
    _stream: OutputStream,
    _sink: Sink,

    pub ch1: AudioSquareChannel,
    pub ch2: AudioSquareChannel,
    pub ch3: AudioCustomChannel,
}

impl Audio {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let sink = Sink::try_new(&stream_handle).unwrap();

        let wave_ch1 = Arc::new(Mutex::new(None));
        let wave_ch2 = Arc::new(Mutex::new(None));
        let wave_ch3 = Arc::new(Mutex::new(None));

        sink.append(MutableWave::new(
            wave_ch1.clone(),
            wave_ch2.clone(),
            wave_ch3.clone(),
        ));

        Self {
            _stream: stream,
            _sink: sink,

            ch1: AudioSquareChannel::new(wave_ch1),
            ch2: AudioSquareChannel::new(wave_ch2),
            ch3: AudioCustomChannel::new(wave_ch3),
        }
    }
}
