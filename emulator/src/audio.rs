use rodio::{OutputStream, Sink, Source};

use std::time::Duration;

const SAMPLE_RATE: u32 = 50000;

#[derive(Clone, Debug)]
pub struct Wave {
    period_value: u16,
    num_sample: usize,
    duty: f32,

    env_initial_volume: f32,
    env_direction: f32,
    env_sweep_pace: u8,
}

impl Wave {
    pub fn new(
        period_value: u16,
        duty: u8,
        env_initial_volume: u8,
        env_direction: u8,
        env_sweep_pace: u8,
    ) -> Wave {
        Wave {
            period_value,
            num_sample: 0,
            duty: [0.125, 0.25, 0.5, 0.75][duty as usize],
            env_initial_volume: env_initial_volume as f32,
            env_direction: if env_direction == 0 { -1. } else { 1. },
            env_sweep_pace,
        }
    }
}

impl Iterator for Wave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);

        if self.period_value == 0 {
            return None;
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

        let sign = if (8. * 32768. / (SAMPLE_RATE as f32) * self.num_sample as f32
            / (2048. - self.period_value as f32))
            % 2.
            > 2. * self.duty
        {
            -1.
        } else {
            1.
        };

        Some(sign * envelope_boundaries / 64.)
    }
}

impl Source for Wave {
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

pub struct AudioChannel {
    _stream: OutputStream,
    sink: Sink,
    pub on: bool,
    pub period_value: u16,
    pub duty: u8,
}

impl AudioChannel {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            sink: Sink::try_new(&stream_handle).unwrap(),
            on: true,
            period_value: 0,
            duty: 0,
        }
    }

    pub fn update(&mut self) {
        if self.on {
            self.sink.stop();
            let source = Wave::new(self.period_value, self.duty, 0xf, 0, 3).amplify(0.25);
            self.sink.append(source);
        } else {
            self.sink.stop();
        }
    }
}

pub struct Audio {
    pub ch1: AudioChannel,
    pub ch2: AudioChannel,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            ch1: AudioChannel::new(),
            ch2: AudioChannel::new(),
        }
    }
}
