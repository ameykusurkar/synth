use std::sync::{Arc, Mutex};
use std::io::{stdin, stdout, Write};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let supported_config = device.default_output_config().expect("no default config!");
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    match sample_format {
        SampleFormat::F32 => run::<f32>(&device, config),
        SampleFormat::I16 => run::<i16>(&device, config),
        SampleFormat::U16 => run::<u16>(&device, config),
    }
}

fn run<T: Sample>(device: &Device, config: StreamConfig) {
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let mut clock = WallClock::new(config.sample_rate.0);
    let num_channels = config.channels as usize;

    let note = Arc::new(Mutex::new(None));
    let stream_note = note.clone();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_samples(data, num_channels, &mut clock, stream_note.clone());
            },
            err_fn,
        )
        .expect("could not build stream");

    stream.play().unwrap();

    let mut stdout = stdout().into_raw_mode().unwrap();
    writeln!(stdout, "Begin playing!{}", termion::cursor::Hide).unwrap();

    let mut mapping = std::collections::HashMap::new();
    mapping.insert('z', 0); // A3
    mapping.insert('s', 1);
    mapping.insert('x', 2);
    mapping.insert('c', 3); // Middle C
    mapping.insert('f', 4);
    mapping.insert('v', 5);
    mapping.insert('g', 6);
    mapping.insert('b', 7);
    mapping.insert('n', 8);
    mapping.insert('j', 9);
    mapping.insert('m', 10);
    mapping.insert('k', 11);
    mapping.insert(',', 12); // A4
    mapping.insert('l', 13);
    mapping.insert('.', 14);
    mapping.insert('/', 15);

    'outer: loop {
        for c in stdin().keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    // Restore cursor state before exiting
                    write!(stdout, "{}", termion::cursor::Show).unwrap();
                    break 'outer;
                }
                Key::Char(c) => {
                    if let Some(semitone) = mapping.get(&c) {
                        let mut n = note.lock().unwrap();
                        let freq = get_freq(*semitone, 220.0);
                        *n = Some(Note(freq));
                    }
                },
                _ => (),
            }
        }
    }
}

fn write_samples<T: Sample>(data: &mut [T], num_channels: usize, clock: &mut WallClock, note: Arc<Mutex<Option<Note>>>) {

    for channel in data.chunks_mut(num_channels) {
        let result = 0.3 * note.lock().unwrap().as_ref().map_or(0.0, |n| n.sample(clock.time()));

        for sample in channel.iter_mut() {
            *sample = Sample::from(&result);
        }

        clock.clock();
    }
}

fn sin(freq: f32, t: f32) -> f32 {
    (2.0 * 3.14159 * freq * t).sin()
}

fn square(freq: f32, t: f32) -> f32 {
    if (2.0 * 3.14159 * freq * t).sin() > 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn get_freq(semitone: u32, root_freq: f32) -> f32 {
    root_freq * twelfth_root(2.0).powf(semitone as f32)
}

fn twelfth_root(x: f32) -> f32 {
    x.sqrt().sqrt().cbrt()
}

struct WallClock {
    sample_rate: f32,
    last_sample_time: f32,
}

impl WallClock {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            last_sample_time: 0.0,
        }
    }

    fn clock(&mut self) {
        self.last_sample_time += 1.0 / self.sample_rate;
    }

    fn time(&self) -> f32 {
        self.last_sample_time
    }
}

struct Note(f32);

impl Note {
    fn sample(&self, t: f32) -> f32 {
        square(self.0, t)
    }
}
