use std::usize;

use crate::audio::track::TrackController;
use crate::audio::{mixer::MixerNode, track::Track};
use crossbeam_channel::{bounded, Receiver, Sender};
use fundsp::hacker32::*;

#[derive(Clone, Copy)]
pub enum AppControllerEnum {
    Play,
    Pause,
    Stop,
    Loop,
    Record(usize),
    Exit,
    SetMixerGain(usize, f32),
    SetMixerReverbMix(usize, f32),
}

pub struct AppController {
    sender: Sender<AppControllerEnum>,
}
impl AppController {
    pub fn new(sender: Sender<AppControllerEnum>) -> Self {
        Self { sender }
    }
    pub fn play(&self) {
        let _ = self.sender.send(AppControllerEnum::Play).unwrap();
    }
    pub fn pause(&self) {
        let _ = self.sender.send(AppControllerEnum::Pause).unwrap();
    }
    pub fn stop(&self) {
        let _ = self.sender.send(AppControllerEnum::Stop).unwrap();
    }
    pub fn set_mixer_gain(&self, track_index: usize, gain: f32) {
        let _ = self
            .sender
            .send(AppControllerEnum::SetMixerGain(track_index, gain));
    }
    pub fn set_mixer_reverb_mix(&self, track_index: usize, mix: f32) {
        let _ = self
            .sender
            .send(AppControllerEnum::SetMixerReverbMix(track_index, mix));
    }
}

pub struct App<const ID: u64> {
    receiver: Receiver<AppControllerEnum>,
    state: AppControllerEnum,
    track_controllers: Vec<TrackController>,
    mixers: Vec<An<MixerNode<ID>>>,
}
impl<const ID: u64> App<ID> {
    pub fn new(
        receiver: Receiver<AppControllerEnum>,
        mixers: Vec<An<MixerNode<ID>>>,
        track_controllers: Vec<TrackController>,
    ) -> Self {
        Self {
            receiver,
            state: AppControllerEnum::Stop,
            track_controllers,
            mixers,
        }
    }
    pub fn set_app_state(&mut self, new_state: AppControllerEnum) {
        self.state = new_state;
    }
    pub fn play(&self) {
        for track_controller in self.track_controllers.iter() {
            track_controller.play();
        }
    }
    pub fn pause(&self) {
        for track_controller in self.track_controllers.iter() {
            track_controller.pause();
        }
    }
    pub fn stop(&self) {
        for track_controller in self.track_controllers.iter() {
            track_controller.stop();
        }
    }
    pub fn record(&self, track_index: usize) {
        if let Some(track) = self.track_controllers.get(track_index) {
            track.record();
        }
    }
    pub fn set_mixer_gain(&self, track_index: usize, gain: f32) {
        if let Some(mixer) = self.mixers.get(track_index) {
            mixer.set_gain(gain);
        }
    }
    pub fn set_mixer_reverb_mix(&self, track_index: usize, mix: f32) {
        if let Some(mixer) = self.mixers.get(track_index) {
            mixer.set_reverb_mix(mix);
        }
    }
}

pub fn build_app<const ID: u64>(
    mixers: Vec<An<MixerNode<ID>>>,
    track_controllers: Vec<TrackController>,
) -> (AppController, App<ID>) {
    let (sender, receiver) = bounded(10);

    let app_controller = AppController::new(sender);

    let app = App::new(receiver, mixers, track_controllers);

    (app_controller, app)
}

pub fn run_app<const ID: u64>(mut app: App<ID>) {
    std::thread::spawn(move || loop {
        if let Ok(msg) = app.receiver.try_recv() {
            app.set_app_state(msg);
            match msg {
                AppControllerEnum::Play => app.play(),
                AppControllerEnum::Pause => app.pause(),
                AppControllerEnum::Record(track_index) => app.record(track_index),
                AppControllerEnum::Stop => app.stop(),
                AppControllerEnum::SetMixerGain(track_index, gain) => {
                    app.set_mixer_gain(track_index, gain)
                }
                AppControllerEnum::SetMixerReverbMix(track_index, mix) => {
                    app.set_mixer_reverb_mix(track_index, mix);
                }
                AppControllerEnum::Loop => {}
                AppControllerEnum::Exit => break,
            }
        }
    });
}
