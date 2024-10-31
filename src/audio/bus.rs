use super::mixer::MixerNode;
use fundsp::hacker32::*;

#[derive(Clone)]
pub struct BusNode<const ID1: u64, const ID2: u64, const ID3: u64, const ID4: u64> {
    sources: (
        An<MixerNode<ID1>>,
        An<MixerNode<ID2>>,
        An<MixerNode<ID3>>,
        An<MixerNode<ID4>>,
    ),
    reverb_mix: Shared,
    gain: Shared,
}
impl<const ID1: u64, const ID2: u64, const ID3: u64, const ID4: u64> BusNode<ID1, ID2, ID3, ID4> {
    pub fn new(
        source1: An<MixerNode<ID1>>,
        source2: An<MixerNode<ID2>>,
        source3: An<MixerNode<ID3>>,
        source4: An<MixerNode<ID4>>,
    ) -> Self {
        Self {
            sources: (source1, source2, source3, source4),
            reverb_mix: Shared::new(0.0),
            gain: Shared::new(1.0),
        }
    }
    pub fn build_graph(&self) -> Option<BlockRateAdapter> {
        let processed_tracks = (
            self.process_source::<ID1>(&self.sources.0),
            self.process_source::<ID2>(&self.sources.1),
            self.process_source::<ID3>(&self.sources.2),
            self.process_source::<ID4>(&self.sources.3),
        );

        let final_source_before_master =
            processed_tracks.0 + processed_tracks.1 + processed_tracks.2 + processed_tracks.3;

        // let master_gain = self.gain.clone();
        // let master_reverb = self.reverb_mix.clone();

        // let gain_node = var(&master_gain) | var(&master_gain);

        // let dry = final_source_before_master.clone() * gain_node.clone() >> join::<U2>();

        // let reverb = reverb2_stereo(20.0, 3.0, 1.0, 0.2, highshelf_hz(1000.0, 1.0, db_amp(-1.0)));
        // let chorus = chorus(0, 0.0, 0.03, 0.2) | chorus(1, 0.0, 0.03, 0.2);

        // let wet = dry.clone() >> reverb >> chorus;

        // let mix_param = var(&master_reverb) | var(&master_reverb);
        // let dry_gain = ((dc(1.0) | dc(1.0)) - mix_param.clone()) * (pass() | pass());

        // let track_graph = (dry * dry_gain.clone() | wet * mix_param.clone()) >> join::<U2>();

        let graph = BlockRateAdapter::new(Box::new(final_source_before_master));

        Some(graph)
    }

    fn process_source<const ID: u64>(&self, source: &An<MixerNode<ID>>) -> Net {
        let track_gain = source.get_gain();
        let track_reverb_mix = source.get_reverb_mix();

        let gain_node = (var(&track_gain) | var(&track_gain));

        let dry = source.clone() * gain_node.clone();

        let reverb = reverb2_stereo(20.0, 3.0, 1.0, 0.2, highshelf_hz(1000.0, 1.0, db_amp(-1.0)));
        let chorus = chorus(0, 0.0, 0.03, 0.2) | chorus(1, 0.0, 0.03, 0.2);

        let wet = dry.clone() >> reverb >> chorus;

        let mix_param = var(&track_reverb_mix) | var(&track_reverb_mix);
        let dry_gain = ((dc(1.0) | dc(1.0)) - mix_param.clone()) * (pass() | pass());

        // Mix dry and wet signals based on reverb_mix
        let track_graph = (dry * dry_gain.clone() | wet * mix_param.clone()) >> join();

        Net::wrap(Box::new(track_graph))
    }
}
