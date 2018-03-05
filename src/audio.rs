#![allow(dead_code)]

use math::*;
use random::*;
use std::f64;

pub struct Waveform {
    pub samples: Box<[f64]>,
    pub sample_count: usize,
    pub sample_rate: f64,
}

impl Waveform {
    pub fn noise(sample_count: usize, sample_rate: f64) -> Waveform {
        let mut data = vec![0.0; sample_count].into_boxed_slice();

        for i in 0..sample_count {
            data[i as usize] = random_pink();
        }

        Waveform {
            samples: data,
            sample_count: sample_count,
            sample_rate: sample_rate,
        }
    }

    pub fn sine(frequency: f64, sample_count: usize, sample_rate: f64) -> Waveform {
        let mut data = vec![0.0; sample_count].into_boxed_slice();

        for i in 0..sample_count {
            data[i] = f64::sin(frequency * (2.0 * PI) * i as f64 / sample_rate);
        }

        Waveform {
            samples: data,
            sample_count: sample_count,
            sample_rate: sample_rate,
        }
    }

    pub fn osc(frequency: f64, sample_count: usize, sample_rate: f64) -> Waveform {
        let initial_phase = 0.0;
        let mut sum = initial_phase;

        let mut data = vec![0.0; sample_count].into_boxed_slice();

        for i in 0..sample_count {
            data[i] = f64::cos(sum) * 0.99;
            let phase_increment = 2.0 * PI * frequency / sample_rate;
            sum = sum + phase_increment;
        }

        Waveform {
            samples: data,
            sample_count: sample_count,
            sample_rate: sample_rate,
        }
    }
}
