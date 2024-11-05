//! Process (stereo) input and play the result (in stereo).

use audio::mixer::MixerNode;
use audio::stream::{build_input_device, build_output_device};
use audio::track::{build_track, run_track};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;
use std::env;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

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

    let reverb = reverb2_stereo(20.0, 3.0, 1.0, 0.2, highshelf_hz(1000.0, 1.0, db_amp(-1.0)));
    let chorus = chorus(0, 0.0, 0.03, 0.2) | chorus(1, 0.0, 0.03, 0.2);

    let mx_one_wet = mixer_one.get_reverb_mix();
    let mx_one_gain = mixer_one.get_gain();

    let mixer_one_processed = mixer_one
        >> ((var(&mx_one_wet) | var(&mx_one_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_one_wet)) | (1.0 - var(&mx_one_wet))) * multipass())
        >> multipass() * (var(&mx_one_gain) | var(&mx_one_gain));

    let mx_two_wet = mixer_two.get_reverb_mix();
    let mx_two_gain = mixer_two.get_gain();

    let mixer_two_processed = mixer_two
        >> ((var(&mx_two_wet) | var(&mx_two_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_two_wet)) | (1.0 - var(&mx_two_wet))) * multipass())
        >> multipass() * (var(&mx_two_gain) | var(&mx_two_gain));

    let mx_three_wet = mixer_three.get_reverb_mix();
    let mx_three_gain = mixer_three.get_gain();

    let mixer_three_processed = mixer_three
        >> ((var(&mx_three_wet) | var(&mx_three_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_three_wet)) | (1.0 - var(&mx_three_wet))) * multipass())
        >> multipass() * (var(&mx_three_gain) | var(&mx_three_gain));

    let mx_four_wet = mixer_four.get_reverb_mix();
    let mx_four_gain = mixer_four.get_gain();

    let mixer_four_processed = mixer_four
        >> ((var(&mx_four_wet) | var(&mx_four_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_four_wet)) | (1.0 - var(&mx_four_wet))) * multipass())
        >> multipass() * (var(&mx_four_gain) | var(&mx_four_gain));

    let mx_five_wet = mixer_five.get_reverb_mix();
    let mx_five_gain = mixer_five.get_gain();

    let mixer_five_processed = mixer_five
        >> ((var(&mx_five_wet) | var(&mx_five_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_five_wet)) | (1.0 - var(&mx_five_wet))) * multipass())
        >> multipass() * (var(&mx_five_gain) | var(&mx_five_gain));

    let mx_six_wet = mixer_six.get_reverb_mix();
    let mx_six_gain = mixer_six.get_gain();

    let mixer_six_processed = mixer_six
        >> ((var(&mx_six_wet) | var(&mx_six_wet)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&mx_six_wet)) | (1.0 - var(&mx_six_wet))) * multipass())
        >> multipass() * (var(&mx_six_gain) | var(&mx_six_gain));

    let master_reverb = shared(0.6);
    let master_gain = shared(0.7);

    let master_bus = (mixer_one_processed
        + mixer_two_processed
        + mixer_three_processed
        + mixer_four_processed
        + mixer_five_processed
        + mixer_six_processed)
        >> ((var(&master_reverb) | var(&master_reverb)) * (reverb.clone() >> chorus.clone())
            & ((1.0 - var(&master_reverb)) | (1.0 - var(&master_reverb))) * multipass())
        >> multipass() * (var(&master_gain) | var(&master_gain));

    run_track(track_one);
    run_track(track_two);
    run_track(track_three);
    run_track(track_four);
    run_track(track_five);
    run_track(track_six);

    build_input_device(sender);

    build_output_device(BlockRateAdapter::new(Box::new(master_bus)));

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
