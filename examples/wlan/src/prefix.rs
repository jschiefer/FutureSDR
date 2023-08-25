use futuresdr::anyhow::Result;
use futuresdr::async_trait;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::ItemTag;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::Tag;
use futuresdr::runtime::WorkIo;

pub struct Prefix {
    pad_front: usize,
    pad_tail: usize,
}

impl Prefix {
    pub fn new(pad_front: usize, pad_tail: usize) -> Block {
        Block::new(
            BlockMetaBuilder::new("Prefix").build(),
            StreamIoBuilder::new()
                .add_input::<Complex32>("in")
                .add_output::<Complex32>("out")
                .build(),
            MessageIoBuilder::new().build(),
            Prefix {
                pad_front,
                pad_tail,
            },
        )
    }
}

#[async_trait]
impl Kernel for Prefix {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _m: &mut MessageIo<Self>,
        _b: &mut BlockMeta,
    ) -> Result<()> {
        let input = sio.input(0).slice::<Complex32>();
        let output = sio.output(0).slice::<Complex32>();

        let tags = sio.input(0).tags().clone();
        if let Some((index, len)) = tags.iter().find_map(|x| match x {
            ItemTag {
                index,
                tag: Tag::NamedUsize(n, len),
            } => {
                if n == "wifi_start" {
                    Some((index, len))
                } else {
                    None
                }
            }
            _ => None,
        }) {
            assert_eq!(*index, 0);
            if output.len() >= self.pad_front + std::cmp::max(self.pad_tail, 1) + len * 80 + 320
                && input.len() >= len * 64
            {
                output[0..self.pad_front].fill(Complex32::new(0.0, 0.0));
                output[self.pad_front..self.pad_front + 320].copy_from_slice(&SYNC_WORDS);

                for k in 0..*len {
                    let in_offset = k * 64;
                    let out_offset = self.pad_front + 320 + k * 80;
                    output[out_offset..out_offset + 16]
                        .copy_from_slice(&input[in_offset + 48..in_offset + 64]);
                    output[out_offset + 16..out_offset + 80]
                        .copy_from_slice(&input[in_offset..in_offset + 64]);
                }

                // windowing
                let out_offset = self.pad_front + 320;
                output[out_offset] = 0.5 * (output[out_offset] + SYNC_WORDS[320 - 64]);
                for k in 0..*len {
                    output[out_offset + (k + 1) * 80] = 0.5
                        * (output[out_offset + (k + 1) * 80] + output[out_offset + k * 80 + 16]);
                }

                let out_offset = self.pad_front + 320 + len * 80;
                output[out_offset + 1..out_offset + std::cmp::max(self.pad_tail, 1)]
                    .fill(Complex32::new(0.0, 0.0));

                sio.input(0).consume(len * 64);
                let produce = self.pad_front + std::cmp::max(self.pad_tail, 1) + len * 80 + 320;

                sio.output(0)
                    .add_tag(0, Tag::NamedUsize("burst_start".to_string(), produce));
                sio.output(0).produce(produce);

                if sio.input(0).finished() && input.len() < len * 64 {
                    io.finished = true;
                }
            }
        } else if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }
}

const SYNC_WORDS: [Complex32; 320] = [
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(0.8164965809277261, 0.0),
    Complex32::new(1.2669822230356942, -0.11228168465644238),
    Complex32::new(-0.11957315586905014, -0.696923425058676),
    Complex32::new(-1.1754648916223063, 0.020764353243054506),
    Complex32::new(0.4082482904638631, 0.4082482904638631),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(0.0, 0.8164965809277261),
    Complex32::new(-0.11228168465644238, 1.2669822230356942),
    Complex32::new(-0.696923425058676, -0.11957315586905014),
    Complex32::new(0.020764353243054506, -1.1754648916223063),
    Complex32::new(-0.4892511000496049, 0.20412414523193154),
    Complex32::new(0.10902823580662051, -0.8662158644642738),
    Complex32::new(0.8140030047248232, -0.9396324876173663),
    Complex32::new(-0.8155207189587305, -1.0217906787851445),
    Complex32::new(-0.024903324538099264, -0.4772575386625251),
    Complex32::new(0.6662943119215744, 0.6571237585016134),
    Complex32::new(-1.1300295588009315, 0.18195391202062006),
    Complex32::new(-1.081771965492788, 0.14702855123982952),
    Complex32::new(-0.3109983073019375, 1.3391647367779653),
    Complex32::new(-0.5010507332532016, 0.19351422463079518),
    Complex32::new(-0.5352643916762729, -0.7214308639580637),
    Complex32::new(0.6173311504866, -0.12533525122045358),
    Complex32::new(0.7297043122370406, -0.8196831598626947),
    Complex32::new(-1.1649823192906783, -0.5789049077694849),
    Complex32::new(-0.5077179404839827, -0.34878295314486224),
    Complex32::new(0.3276542347555771, -0.8728243113896111),
    Complex32::new(0.5547001962252291, 0.5547001962252291),
    Complex32::new(1.0582711327103833, 0.036349232408789636),
    Complex32::new(-0.199543023211113, -1.4258664658215698),
    Complex32::new(0.5206972262176459, 0.1325866548273851),
    Complex32::new(0.21722815426455683, 0.5194815769152705),
    Complex32::new(-1.2141710722292116, 0.4205054505865139),
    Complex32::new(0.008777395818215522, 1.0206895741609938),
    Complex32::new(0.47338322757681417, -0.036178225577857305),
    Complex32::new(0.8656985035271666, 0.22976434432750706),
    Complex32::new(-0.340061994825911, 0.9422884169761446),
    Complex32::new(-1.0218129188969418, 0.48973810579149746),
    Complex32::new(0.5309487757158858, 0.7784153102584617),
    Complex32::new(0.18737125048696027, -0.24749319433501804),
    Complex32::new(0.8594026461369145, -0.7348482662383934),
    Complex32::new(0.35278664762528644, 0.986549325159579),
    Complex32::new(-0.04545213727749439, 1.0679099952790945),
    Complex32::new(1.386750490563073, 0.0),
    Complex32::new(-0.04545213727749439, -1.0679099952790942),
    Complex32::new(0.35278664762528633, -0.986549325159579),
    Complex32::new(0.8594026461369145, 0.7348482662383935),
    Complex32::new(0.18737125048696027, 0.24749319433501804),
    Complex32::new(0.5309487757158858, -0.7784153102584619),
    Complex32::new(-1.0218129188969418, -0.4897381057914974),
    Complex32::new(-0.34006199482591115, -0.9422884169761446),
    Complex32::new(0.8656985035271666, -0.22976434432750706),
    Complex32::new(0.47338322757681417, 0.03617822557785712),
    Complex32::new(0.008777395818215522, -1.020689574160994),
    Complex32::new(-1.2141710722292116, -0.4205054505865138),
    Complex32::new(0.21722815426455683, -0.5194815769152705),
    Complex32::new(0.5206972262176458, -0.1325866548273849),
    Complex32::new(-0.19954302321111303, 1.4258664658215698),
    Complex32::new(1.0582711327103833, -0.036349232408789754),
    Complex32::new(0.5547001962252291, -0.5547001962252291),
    Complex32::new(0.3276542347555771, 0.8728243113896109),
    Complex32::new(-0.5077179404839827, 0.34878295314486224),
    Complex32::new(-1.1649823192906783, 0.5789049077694848),
    Complex32::new(0.7297043122370406, 0.8196831598626947),
    Complex32::new(0.6173311504865998, 0.12533525122045355),
    Complex32::new(-0.5352643916762727, 0.7214308639580638),
    Complex32::new(-0.5010507332532016, -0.19351422463079507),
    Complex32::new(-0.3109983073019375, -1.3391647367779653),
    Complex32::new(-1.081771965492788, -0.14702855123982952),
    Complex32::new(-1.1300295588009313, -0.1819539120206201),
    Complex32::new(0.6662943119215742, -0.6571237585016134),
    Complex32::new(-0.024903324538099264, 0.4772575386625251),
    Complex32::new(-0.8155207189587307, 1.0217906787851443),
    Complex32::new(0.8140030047248235, 0.9396324876173663),
    Complex32::new(0.1090282358066204, 0.866215864464274),
    Complex32::new(-1.386750490563073, 0.0),
    Complex32::new(0.10902823580662051, -0.8662158644642738),
    Complex32::new(0.8140030047248232, -0.9396324876173663),
    Complex32::new(-0.8155207189587305, -1.0217906787851445),
    Complex32::new(-0.024903324538099264, -0.4772575386625251),
    Complex32::new(0.6662943119215744, 0.6571237585016134),
    Complex32::new(-1.1300295588009315, 0.18195391202062006),
    Complex32::new(-1.081771965492788, 0.14702855123982952),
    Complex32::new(-0.3109983073019375, 1.3391647367779653),
    Complex32::new(-0.5010507332532016, 0.19351422463079518),
    Complex32::new(-0.5352643916762729, -0.7214308639580637),
    Complex32::new(0.6173311504866, -0.12533525122045358),
    Complex32::new(0.7297043122370406, -0.8196831598626947),
    Complex32::new(-1.1649823192906783, -0.5789049077694849),
    Complex32::new(-0.5077179404839827, -0.34878295314486224),
    Complex32::new(0.3276542347555771, -0.8728243113896111),
    Complex32::new(0.5547001962252291, 0.5547001962252291),
    Complex32::new(1.0582711327103833, 0.036349232408789636),
    Complex32::new(-0.199543023211113, -1.4258664658215698),
    Complex32::new(0.5206972262176459, 0.1325866548273851),
    Complex32::new(0.21722815426455683, 0.5194815769152705),
    Complex32::new(-1.2141710722292116, 0.4205054505865139),
    Complex32::new(0.008777395818215522, 1.0206895741609938),
    Complex32::new(0.47338322757681417, -0.036178225577857305),
    Complex32::new(0.8656985035271666, 0.22976434432750706),
    Complex32::new(-0.340061994825911, 0.9422884169761446),
    Complex32::new(-1.0218129188969418, 0.48973810579149746),
    Complex32::new(0.5309487757158858, 0.7784153102584617),
    Complex32::new(0.18737125048696027, -0.24749319433501804),
    Complex32::new(0.8594026461369145, -0.7348482662383934),
    Complex32::new(0.35278664762528644, 0.986549325159579),
    Complex32::new(-0.04545213727749439, 1.0679099952790945),
    Complex32::new(1.386750490563073, 0.0),
    Complex32::new(-0.04545213727749439, -1.0679099952790942),
    Complex32::new(0.35278664762528633, -0.986549325159579),
    Complex32::new(0.8594026461369145, 0.7348482662383935),
    Complex32::new(0.18737125048696027, 0.24749319433501804),
    Complex32::new(0.5309487757158858, -0.7784153102584619),
    Complex32::new(-1.0218129188969418, -0.4897381057914974),
    Complex32::new(-0.34006199482591115, -0.9422884169761446),
    Complex32::new(0.8656985035271666, -0.22976434432750706),
    Complex32::new(0.47338322757681417, 0.03617822557785712),
    Complex32::new(0.008777395818215522, -1.020689574160994),
    Complex32::new(-1.2141710722292116, -0.4205054505865138),
    Complex32::new(0.21722815426455683, -0.5194815769152705),
    Complex32::new(0.5206972262176458, -0.1325866548273849),
    Complex32::new(-0.19954302321111303, 1.4258664658215698),
    Complex32::new(1.0582711327103833, -0.036349232408789754),
    Complex32::new(0.5547001962252291, -0.5547001962252291),
    Complex32::new(0.3276542347555771, 0.8728243113896109),
    Complex32::new(-0.5077179404839827, 0.34878295314486224),
    Complex32::new(-1.1649823192906783, 0.5789049077694848),
    Complex32::new(0.7297043122370406, 0.8196831598626947),
    Complex32::new(0.6173311504865998, 0.12533525122045355),
    Complex32::new(-0.5352643916762727, 0.7214308639580638),
    Complex32::new(-0.5010507332532016, -0.19351422463079507),
    Complex32::new(-0.3109983073019375, -1.3391647367779653),
    Complex32::new(-1.081771965492788, -0.14702855123982952),
    Complex32::new(-1.1300295588009313, -0.1819539120206201),
    Complex32::new(0.6662943119215742, -0.6571237585016134),
    Complex32::new(-0.024903324538099264, 0.4772575386625251),
    Complex32::new(-0.8155207189587307, 1.0217906787851443),
    Complex32::new(0.8140030047248235, 0.9396324876173663),
    Complex32::new(0.1090282358066204, 0.866215864464274),
    Complex32::new(-1.386750490563073, 0.0),
    Complex32::new(0.10902823580662051, -0.8662158644642738),
    Complex32::new(0.8140030047248232, -0.9396324876173663),
    Complex32::new(-0.8155207189587305, -1.0217906787851445),
    Complex32::new(-0.024903324538099264, -0.4772575386625251),
    Complex32::new(0.6662943119215744, 0.6571237585016134),
    Complex32::new(-1.1300295588009315, 0.18195391202062006),
    Complex32::new(-1.081771965492788, 0.14702855123982952),
    Complex32::new(-0.3109983073019375, 1.3391647367779653),
    Complex32::new(-0.5010507332532016, 0.19351422463079518),
    Complex32::new(-0.5352643916762729, -0.7214308639580637),
    Complex32::new(0.6173311504866, -0.12533525122045358),
    Complex32::new(0.7297043122370406, -0.8196831598626947),
    Complex32::new(-1.1649823192906783, -0.5789049077694849),
    Complex32::new(-0.5077179404839827, -0.34878295314486224),
    Complex32::new(0.3276542347555771, -0.8728243113896111),
    Complex32::new(0.5547001962252291, 0.5547001962252291),
    Complex32::new(1.0582711327103833, 0.036349232408789636),
    Complex32::new(-0.199543023211113, -1.4258664658215698),
    Complex32::new(0.5206972262176459, 0.1325866548273851),
    Complex32::new(0.21722815426455683, 0.5194815769152705),
    Complex32::new(-1.2141710722292116, 0.4205054505865139),
    Complex32::new(0.008777395818215522, 1.0206895741609938),
    Complex32::new(0.47338322757681417, -0.036178225577857305),
    Complex32::new(0.8656985035271666, 0.22976434432750706),
    Complex32::new(-0.340061994825911, 0.9422884169761446),
    Complex32::new(-1.0218129188969418, 0.48973810579149746),
    Complex32::new(0.5309487757158858, 0.7784153102584617),
    Complex32::new(0.18737125048696027, -0.24749319433501804),
    Complex32::new(0.8594026461369145, -0.7348482662383934),
    Complex32::new(0.35278664762528644, 0.986549325159579),
    Complex32::new(-0.04545213727749439, 1.0679099952790945),
];
