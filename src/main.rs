//! Process (stereo) input and play the result (in stereo).

use audio::audio_graph::build_audio_graph;
use audio::mixer::MixerNode;
use audio::stream::{build_input_device, build_output_device};
use audio::track::{build_track, run_track};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;
use std::env;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

mod app;
mod audio;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    // Sender / receiver for left and right channels (stereo mic).
    let (sender, receiver) = bounded(4096);

    let (track_one_controller, track_one, track_one_receiver) = build_track(receiver.clone());
    let (track_two_controller, track_two, track_two_receiver) = build_track(receiver.clone());
    let (track_three_controller, track_three, track_three_receiver) = build_track(receiver.clone());
    let (track_four_controller, track_four, track_four_receiver) = build_track(receiver.clone());
    let (track_five_controller, track_five, track_five_receiver) = build_track(receiver.clone());
    let (track_six_controller, track_six, track_six_receiver) = build_track(receiver.clone());

    let mixer_one = An(MixerNode::<1>::new(track_one_receiver));
    let mixer_two = An(MixerNode::<2>::new(track_two_receiver));
    let mixer_three = An(MixerNode::<3>::new(track_three_receiver));
    let mixer_four = An(MixerNode::<4>::new(track_four_receiver));
    let mixer_five = An(MixerNode::<5>::new(track_five_receiver));
    let mixer_six = An(MixerNode::<6>::new(track_six_receiver));
    // let master_bus = BusNode::new(mixer_one, mixer_two, mixer_three, mixer_four);

    run_track(track_one);
    run_track(track_two);
    run_track(track_three);
    run_track(track_four);
    run_track(track_five);
    run_track(track_six);

    let master_bus = build_audio_graph(
        mixer_one.clone(),
        mixer_two.clone(),
        mixer_three.clone(),
        mixer_four.clone(),
        mixer_five.clone(),
        mixer_six.clone(),
    );

    build_input_device(sender);

    build_output_device(BlockRateAdapter::new(master_bus));

    track_one_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_one_controller.play();

    std::thread::sleep(Duration::from_millis(5));

    track_two_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_two_controller.play();

    std::thread::sleep(Duration::from_millis(5));

    track_three_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_three_controller.play();

    std::thread::sleep(Duration::from_millis(5));

    track_four_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_four_controller.play();

    std::thread::sleep(Duration::from_millis(5));

    track_five_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_six_controller.only_input();

    println!("Processing stereo input to stereo output.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
