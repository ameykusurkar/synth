use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};

use druid::{lens, widget};
use druid::{
    AppLauncher, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LensExt, LifeCycle,
    LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget, WidgetExt, WindowDesc,
};

use crate::envelope::Envelope;
use crate::in_arc_mutex::InArcMutex;
use crate::note::Note;

mod envelope;
mod in_arc_mutex;
mod note;

lazy_static! {
    static ref KEY_MAPPING: HashMap<char, f32> = build_keyboard();
}

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

    let clock = Arc::new(Mutex::new(WallClock::new(config.sample_rate.0)));
    let num_channels = config.channels as usize;

    let notes = Arc::new(Mutex::new(HashMap::new()));
    let stream_notes = notes.clone();
    let stream_clock = clock.clone();

    let envelope = Arc::new(Mutex::new(ENVELOPE.clone()));
    let stream_envelope = envelope.clone();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_samples(
                    data,
                    num_channels,
                    stream_envelope.clone(),
                    stream_clock.clone(),
                    stream_notes.clone(),
                );
            },
            err_fn,
        )
        .expect("could not build stream");

    stream.play().unwrap();

    let launcher = AppLauncher::with_window(WindowDesc::new(build_ui));
    launcher
        .launch(KeyboardState::new(
            notes.clone(),
            clock.clone(),
            envelope.clone(),
        ))
        .unwrap();
}

fn build_ui() -> impl Widget<KeyboardState> {
    let arc_envelope_lens =
        InArcMutex::new(lens!(Envelope, attack_duration).map(|x| *x as f64, |x, y| *x = y as f32));

    let lens = druid::lens::Id
        .then(lens!(KeyboardState, envelope))
        .then(arc_envelope_lens);

    let slider = widget::Slider::new().with_range(0.0, 2.0).lens(lens);
    let label = widget::Label::new(|data: &KeyboardState, _env: &_| {
        format!(
            "Attack Duration: {}",
            data.envelope
                .clone()
                .as_ref()
                .lock()
                .unwrap()
                .attack_duration
        )
    });

    widget::Flex::row().with_flex_child(
        widget::Flex::column()
            .with_flex_child(label, 1.0)
            .with_flex_child(slider, 1.0)
            .with_flex_child(Keyboard, 1.0),
        1.0,
    )
}

fn write_samples<T: Sample>(
    data: &mut [T],
    num_channels: usize,
    envelope: Arc<Mutex<Envelope>>,
    clock: Arc<Mutex<WallClock>>,
    notes: Arc<Mutex<HashMap<char, Note>>>,
) {
    for channel in data.chunks_mut(num_channels) {
        let mut clock = clock.lock().unwrap();
        let mut result = 0.0;

        let envelope = envelope.lock().unwrap();

        for (_, note) in notes.lock().unwrap().iter_mut() {
            result += 0.1 * note.sample(clock.time(), &envelope);
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

struct Keyboard;

#[derive(Clone, Data, Lens)]
struct KeyboardState {
    notes: Arc<Mutex<HashMap<char, Note>>>,
    clock: Arc<Mutex<WallClock>>,
    envelope: Arc<Mutex<Envelope>>,
}

impl KeyboardState {
    fn new(
        notes: Arc<Mutex<HashMap<char, Note>>>,
        clock: Arc<Mutex<WallClock>>,
        envelope: Arc<Mutex<Envelope>>,
    ) -> Self {
        Self {
            notes,
            clock,
            envelope,
        }
    }
}

const ENVELOPE: Envelope = Envelope {
    attack_duration: 0.1,
    attack_amplitude: 1.0,
    decay_duration: 0.1,
    sustain_amplitude: 0.9,
    release_duration: 0.2,
};

impl Widget<KeyboardState> for Keyboard {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut KeyboardState, _env: &Env) {
        match event {
            Event::WindowConnected => {
                // This is the only widget, so it should handle all keyboard events
                ctx.request_focus();
            }
            Event::KeyDown(k) => {
                if !k.is_repeat {
                    let key = k
                        .unmod_text()
                        .map_or(' ', |s| s.chars().next().unwrap_or(' '));

                    if let Some(freq) = KEY_MAPPING.get(&key) {
                        let t = data.clock.lock().unwrap().time();
                        data.notes.lock().unwrap().insert(key, Note::new(*freq, t));
                    }
                }
            }
            Event::KeyUp(k) => {
                let key = k
                    .unmod_text()
                    .map_or(' ', |s| s.chars().next().unwrap_or(' '));

                let t = data.clock.lock().unwrap().time();
                data.notes
                    .lock()
                    .unwrap()
                    .get_mut(&key)
                    .map(|n| n.release(t));
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &KeyboardState,
        _: &Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &KeyboardState,
        _data: &KeyboardState,
        _: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _: &mut LayoutCtx,
        bc: &BoxConstraints,
        _: &KeyboardState,
        _: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &KeyboardState, _env: &Env) {}
}
