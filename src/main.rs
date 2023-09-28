//! Shows how to create a custom [`Decodable`] type by implementing a Sine wave.

use std::sync::{Arc, Mutex};
use bevy::audio::AddAudioSource;
use bevy::audio::AudioPlugin;
use bevy::audio::Source;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::Duration;
use bevy::window::PrimaryWindow;

struct SynthParams {
    frequency: f32,
    volume: f32,
    distortion: f32,
}

#[derive(TypePath, TypeUuid)]
#[uuid = "c2090c23-78fd-44f1-8508-c89b1f3cec29"]
struct SynthAudio {
    params: Arc<Mutex<SynthParams>>,
}

struct SineDecoder {
    params: Arc<Mutex<SynthParams>>,
    // how far along one period the wave is (between 0 and 1)
    current_progress: f32,
    // how much we move along the period every frame
    // how long a period is
    period: f32,
    sample_rate: u32,
}

impl SineDecoder {
    fn new(audio: &SynthAudio) -> Self {
        SineDecoder {
            params: audio.params.clone(),
            current_progress: 0.,
            period: std::f32::consts::PI * 2.,
            sample_rate: 44_100,
        }
    }
}

// The decoder must implement iterator so that it can implement `Decodable`.
impl Iterator for SineDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_progress += self.params.lock().unwrap().frequency / self.sample_rate as f32;
        // we loop back round to 0 to avoid floating point inaccuracies
        self.current_progress %= 1.;
        let act_dist = (self.params.lock().unwrap().distortion + 1.0) * 2.0;
        let vol = self.params.lock().unwrap().volume;

        Some(f32::clamp(f32::sin(self.period * self.current_progress) * act_dist, -1.0, 1.0) * vol)
    }
}
// `Source` is what allows the audio source to be played by bevy.
// This trait provides information on the audio.
impl Source for SineDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for SynthAudio {
    type DecoderItem = <SineDecoder as Iterator>::Item;

    type Decoder = SineDecoder;

    fn decoder(&self) -> Self::Decoder {
        SineDecoder::new(self)
    }
}

fn main() {
    let mut app = App::new();
    // register the audio source so that it can be used
    app.add_plugins(DefaultPlugins.set(AudioPlugin {
            global_volume: GlobalVolume::new(0.4),
        }))
        .add_audio_source::<SynthAudio>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_freq)
        .run();
}

fn setup(mut assets: ResMut<Assets<SynthAudio>>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // add a `SineAudio` to the asset server so that it can be played
    let audio_handle = assets.add(SynthAudio {
        params: Arc::new(Mutex::new(SynthParams {
            frequency: 440.0,
            volume: 0.5,
            distortion: 0.0,
        }))
    });

    commands.spawn(AudioSourceBundle {
        source: audio_handle,
        ..default()
    });
}

fn update_freq(q_windows: Query<&Window, With<PrimaryWindow>>,
               mut assets: ResMut<Assets<SynthAudio>>,
               handle_query: Query<&Handle<SynthAudio>>,
               time: Res<Time>) {

    if let Some(position) = q_windows.single().cursor_position() {

        let handle = handle_query.single();
        assets
            .get_mut(handle)
            .unwrap()
            .params.lock().unwrap()
            .frequency = (1.0 - position.y / q_windows.single().resolution.height()) * 880.0;
        assets
            .get_mut(handle)
            .unwrap()
            .params.lock().unwrap()
            .volume = 0.5 + position.y / q_windows.single().resolution.height();
        assets
            .get_mut(handle)
            .unwrap()
            .params.lock().unwrap()
            .distortion = 1.0 - ((position.x / q_windows.single().resolution.width()) - 0.5).abs() * 2.0;
    }

}
