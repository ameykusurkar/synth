use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};

use druid::{
    AppLauncher, WindowDesc, Widget, PlatformError,
    LifeCycle, LifeCycleCtx, Env, Event, EventCtx,
    UpdateCtx, PaintCtx, LayoutCtx, BoxConstraints, Size, Data
};

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

    let notes = Arc::new(Mutex::new(HashMap::new()));
    let stream_notes = notes.clone();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_samples(data, num_channels, &mut clock, stream_notes.clone());
            },
            err_fn,
        )
        .expect("could not build stream");

    stream.play().unwrap();

    let mapping = build_keyboard();

    let launcher = AppLauncher::with_window(WindowDesc::new(build_ui));
    launcher.launch(KeyboardState::new(notes.clone())).unwrap();
}

fn build_ui() -> Keyboard {
    Keyboard
}

fn write_samples<T: Sample>(
    data: &mut [T],
    num_channels: usize,
    clock: &mut WallClock,
    notes: Arc<Mutex<HashMap<char, Note>>>,
) {
    for channel in data.chunks_mut(num_channels) {
        let mut result = 0.0;

        for (_, note) in notes.lock().unwrap().iter_mut() {
            result += 0.1
                * note
                    .sample(clock.time(), 1.0 / clock.sample_rate)
                    .unwrap_or(0.0);
        }

        for sample in channel.iter_mut() {
            *sample = Sample::from(&result);
        }

        clock.clock();
    }
}

fn build_keyboard() -> HashMap<char, f32> {
    let root_freq = 220.0;
    let mut mapping = HashMap::new();

    mapping.insert('z', get_freq(0, root_freq)); // A3
    mapping.insert('s', get_freq(1, root_freq));
    mapping.insert('x', get_freq(2, root_freq));
    mapping.insert('c', get_freq(3, root_freq)); // Middle C
    mapping.insert('f', get_freq(4, root_freq));
    mapping.insert('v', get_freq(5, root_freq));
    mapping.insert('g', get_freq(6, root_freq));
    mapping.insert('b', get_freq(7, root_freq));
    mapping.insert('n', get_freq(8, root_freq));
    mapping.insert('j', get_freq(9, root_freq));
    mapping.insert('m', get_freq(10, root_freq));
    mapping.insert('k', get_freq(11, root_freq));
    mapping.insert(',', get_freq(12, root_freq)); // A4
    mapping.insert('l', get_freq(13, root_freq));
    mapping.insert('.', get_freq(14, root_freq));
    mapping.insert('/', get_freq(15, root_freq));

    mapping
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

fn sawtooth(freq: f32, t: f32) -> f32 {
    let period = 1.0 / freq;
    2.0 * (t / period - (0.5 + t / period).floor())
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

struct Note {
    freq: f32,
    time_remaining: f32,
}

impl Note {
    fn new(freq: f32, duration: f32) -> Self {
        Self {
            freq,
            time_remaining: duration,
        }
    }

    fn sample(&mut self, t: f32, sample_duration: f32) -> Option<f32> {
        if self.time_remaining > 0.0 {
            self.time_remaining -= sample_duration;
            Some(sawtooth(self.freq, t))
        } else {
            None
        }
    }
}

struct Keyboard;

#[derive(Clone, Data)]
struct KeyboardState {
    notes: Arc<Mutex<HashMap<char, Note>>>,
}

impl KeyboardState {
    fn new(notes: Arc<Mutex<HashMap<char, Note>>>) -> Self {
        Self { notes }
    }
}

impl Widget<KeyboardState> for Keyboard {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut KeyboardState, _env: &Env) {
        match event {
            Event::WindowConnected => {
                // This is the only widget, so it should handle all keyboard events
                ctx.request_focus();
            }
            Event::KeyDown(k) => {
                // if let Some(freq) = mapping.get(&c) {
                //     notes.lock().unwrap().insert(c, Note::new(*freq, 0.3));
                // }
                let freq = 440.0;
                let key = k.unmod_text().map_or(' ', |s| s.chars().next().unwrap_or(' '));
                println!("{}, {}", key, freq);
                data.notes.lock().unwrap().insert(key, Note::new(freq, 0.3));
            }
            // Event::KeyUp(k) => {
            //     println!("Up {:?}", k);
            // }
            // e => println!("Event {:?}", e),
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &KeyboardState, _: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &KeyboardState, data: &KeyboardState, _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &KeyboardState, _: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &KeyboardState, _env: &Env) {}
}
