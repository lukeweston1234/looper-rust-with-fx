//! Process (stereo) input and play the result (in stereo).

use std::time::Duration;

use audio::bus::BusNode;
use audio::mixer::MixerNode;
use audio::track::{build_track, run_track};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;
use std::env;

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

    let master_bus = (mixer_one + mixer_two + mixer_three + mixer_four + mixer_five + mixer_six)
        >> reverb2_stereo(20.0, 3.0, 1.0, 0.2, highshelf_hz(1000.0, 1.0, db_amp(-1.0)))
        >> (chorus(0, 0.0, 0.03, 0.2) | chorus(1, 0.0, 0.03, 0.2));

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

    track_two_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_three_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_four_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_five_controller.record();

    std::thread::sleep(Duration::from_secs(8));

    track_six_controller.only_input();

    println!("Processing stereo input to stereo output.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub fn build_output_device(mut master_bus: BlockRateAdapter) {
    let host = cpal::default_host();

    // Start output.
    let out_device = host.default_output_device().unwrap();
    let out_config = out_device.default_output_config().unwrap();
    match out_config.sample_format() {
        cpal::SampleFormat::F32 => run_out::<f32>(&out_device, &out_config.into(), master_bus),
        cpal::SampleFormat::I16 => run_out::<i16>(&out_device, &out_config.into(), master_bus),
        cpal::SampleFormat::U16 => run_out::<u16>(&out_device, &out_config.into(), master_bus),
        format => eprintln!("Unsupported sample format: {}", format),
    }
}

pub fn build_input_device(sender: Sender<(f32, f32)>) {
    let host = cpal::default_host();
    // Start input.
    let in_device = host.default_input_device().unwrap();
    let in_config = in_device.default_input_config().unwrap();
    println!("{}", in_config.channels());
    match in_config.sample_format() {
        cpal::SampleFormat::F32 => run_in::<f32>(&in_device, &in_config.into(), sender),
        cpal::SampleFormat::I16 => run_in::<i16>(&in_device, &in_config.into(), sender),
        cpal::SampleFormat::U16 => run_in::<u16>(&in_device, &in_config.into(), sender),
        format => eprintln!("Unsupported sample format: {}", format),
    }
}

#[derive(Clone)]
pub struct InputNode {
    receiver: Receiver<(f32, f32)>,
}

impl InputNode {
    pub fn new(receiver: Receiver<(f32, f32)>) -> Self {
        InputNode { receiver }
    }
}

impl AudioNode for InputNode {
    const ID: u64 = 87;
    type Inputs = U0;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let (left, right) = self.receiver.try_recv().unwrap_or((0.0, 0.0));
        [left, right].into()
    }
}

fn run_in<T>(device: &cpal::Device, config: &cpal::StreamConfig, sender: Sender<(f32, f32)>)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| read_data(data, channels, sender.clone()),
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            std::mem::forget(stream);
        }
    }
    println!("Input stream built.");
}

fn read_data<T>(input: &[T], channels: usize, sender: Sender<(f32, f32)>)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    for frame in input.chunks(channels) {
        let mut left = 0.0;
        let mut right = 0.0;
        for (channel, sample) in frame.iter().enumerate() {
            if channel & 1 == 0 {
                left = sample.to_sample::<f32>();
            } else {
                right = sample.to_sample::<f32>();
            }
        }
        if let Ok(()) = sender.try_send((left, right)) {}
    }
}

fn run_out<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut bus: BlockRateAdapter)
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    bus.set_sample_rate(config.sample_rate.0 as f64);

    let mut next_value = move || bus.get_stereo();

    let err_fn = |err| eprintln!("An error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            std::mem::forget(stream);
        }
    }
    println!("Output stream built.");
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
