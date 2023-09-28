//! Shows how to create a custom [`Decodable`] type by implementing a Sine wave.

use bevy::audio::AddAudioSource;
use bevy::audio::AudioPlugin;
use bevy::audio::Source;
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::Duration;

#[derive(TypePath, TypeUuid)]
#[uuid = "c2090c23-78fd-44f1-8508-c89b1f3cec29"]
struct SynthParams {
    frequency: f32,
    volume: f32
}

struct SineDecoder<'a> {
    params: &'a SynthParams,
    // how far along one period the wave is (between 0 and 1)
    current_progress: f32,
    // how much we move along the period every frame
    // how long a period is
    period: f32,
    sample_rate: u32,
}

impl SineDecoder<'_> {
    fn new<'a>(params: &'a SynthParams) -> Self {
        SineDecoder {
            params,
            current_progress: 0.,
            period: std::f32::consts::PI * 2.,
            sample_rate: 44_100,
        }
    }
}

// The decoder must implement iterator so that it can implement `Decodable`.
impl Iterator for SineDecoder<'_> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_progress += self.params.frequency / self.sample_rate as f32;
        // we loop back round to 0 to avoid floating point inaccuracies
        self.current_progress %= 1.;
        Some(f32::sin(self.period * self.current_progress) * self.params.volume)
    }
}
// `Source` is what allows the audio source to be played by bevy.
// This trait provides information on the audio.
impl Source for SineDecoder<'_> {
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

impl Decodable for SynthParams {
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
            global_volume: GlobalVolume::new(0.2),
        }))
        .add_audio_source::<SynthParams>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut assets: ResMut<Assets<SynthParams>>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // add a `SineAudio` to the asset server so that it can be played
    let audio_handle = assets.add(SynthParams {
        frequency: 440., // this is the frequency of A4
        volume: 1.,
    });
    commands.spawn(AudioSourceBundle {
        source: audio_handle,
        ..default()
    });
}

