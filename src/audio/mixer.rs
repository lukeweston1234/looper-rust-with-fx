use crossbeam_channel::Receiver;
use fundsp::hacker32::*;

#[derive(Clone)]
pub struct MixerNode<const ID: u64> {
    receiver: Receiver<(f32, f32)>,
    reverb_mix: Shared,
    gain: Shared,
}

impl<const ID: u64> MixerNode<ID> {
    pub fn new(receiver: Receiver<(f32, f32)>) -> Self {
        Self {
            receiver,
            reverb_mix: shared(0.0),
            gain: shared(1.0),
        }
    }
    pub fn get_gain(&self) -> Shared {
        self.gain.clone()
    }

    pub fn get_reverb_mix(&self) -> Shared {
        self.reverb_mix.clone()
    }
}

impl<const ID: u64> AudioNode for MixerNode<ID> {
    const ID: u64 = ID;
    type Inputs = U0;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, _: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let (left, right) = self.receiver.try_recv().unwrap_or((0.0, 0.0));
        [left, right].into()
    }
}
