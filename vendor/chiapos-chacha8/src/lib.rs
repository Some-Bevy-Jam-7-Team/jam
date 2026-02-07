use rayon::prelude::*;

#[derive(Default)]
pub struct ChaCha8 {
    input: [u32; 16],
}

const SIGMA: [u8; 16] = *b"expand 32-byte k";
// const TAU: [u8; 16] = *b"expand 16-byte k";

#[inline]
const fn u8to32_idx_little(i: u8, idx: u8) -> u32 {
    (i as u32).to_le() << (24 - idx * 8)
}

#[inline]
const fn u8sto32_little(i1: u8, i2: u8, i3: u8, i4: u8) -> u32 {
    u8to32_idx_little(i1, 3)
        | u8to32_idx_little(i2, 2)
        | u8to32_idx_little(i3, 1)
        | u8to32_idx_little(i4, 0)
}

#[inline]
fn u32to8_little(i: u32, o: &mut [u8]) {
    let a = i.to_le_bytes();
    o[0..4].copy_from_slice(&a);
}

#[inline]
const fn rotl32(v: u32, n: u32) -> u32 {
    (v.to_le() << n.to_le()) | (v.to_le() >> (32_u32.to_le().wrapping_sub(n.to_le())))
}

#[inline]
const fn rotate(v: u32, c: u32) -> u32 {
    rotl32(v, c)
}

#[inline]
const fn xor(v: u32, w: u32) -> u32 {
    v.to_le() ^ w.to_le()
}

#[inline]
const fn plus(v: u32, w: u32) -> u32 {
    v.wrapping_add(w)
}

macro_rules! quarter_round {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        $a = plus($a, $b);
        $d = rotate(xor($d, $a), 16);
        $c = plus($c, $d);
        $b = rotate(xor($b, $c), 12);
        $a = plus($a, $b);
        $d = rotate(xor($d, $a), 8);
        $c = plus($c, $d);
        $b = rotate(xor($b, $c), 7);
    };
}

impl ChaCha8 {
    pub fn new_from_256bit_key(k: &[u8; 32]) -> Self {
        let mut x = Self::default();

        x.input[0] = u8sto32_little(SIGMA[0], SIGMA[1], SIGMA[2], SIGMA[3]);
        x.input[1] = u8sto32_little(SIGMA[4], SIGMA[5], SIGMA[6], SIGMA[7]);
        x.input[2] = u8sto32_little(SIGMA[8], SIGMA[9], SIGMA[10], SIGMA[11]);
        x.input[3] = u8sto32_little(SIGMA[12], SIGMA[13], SIGMA[14], SIGMA[15]);
        x.input[4] = u8sto32_little(k[0], k[1], k[2], k[3]);
        x.input[5] = u8sto32_little(k[4], k[5], k[6], k[7]);
        x.input[6] = u8sto32_little(k[8], k[9], k[10], k[11]);
        x.input[7] = u8sto32_little(k[12], k[13], k[14], k[15]);
        x.input[8] = u8sto32_little(k[16], k[17], k[18], k[19]);
        x.input[9] = u8sto32_little(k[20], k[21], k[22], k[23]);
        x.input[10] = u8sto32_little(k[24], k[25], k[26], k[27]);
        x.input[11] = u8sto32_little(k[28], k[29], k[30], k[31]);

        // IV is always None
        // x.input[14] = 0;
        // x.input[15] = 0;

        x
    }

    // pub fn new_from_128bit_key(k: &[u8; 32]) -> Self {
    //     let mut x = Self::default();
    //
    //     x.input[0] = u8sto32_little(TAU[0], TAU[1], TAU[2], TAU[3]);
    //     x.input[1] = u8sto32_little(TAU[4], TAU[5], TAU[6], TAU[7]);
    //     x.input[2] = u8sto32_little(TAU[8], TAU[9], TAU[10], TAU[11]);
    //     x.input[3] = u8sto32_little(TAU[12], TAU[13], TAU[14], TAU[15]);
    //     x.input[4] = u8sto32_little(k[0], k[1], k[2], k[3]);
    //     x.input[5] = u8sto32_little(k[4], k[5], k[6], k[7]);
    //     x.input[6] = u8sto32_little(k[8], k[9], k[10], k[11]);
    //     x.input[7] = u8sto32_little(k[12], k[13], k[14], k[15]);
    //     x.input[8] = u8sto32_little(k[16], k[17], k[18], k[19]);
    //     x.input[9] = u8sto32_little(k[20], k[21], k[22], k[23]);
    //     x.input[10] = u8sto32_little(k[24], k[25], k[26], k[27]);
    //     x.input[11] = u8sto32_little(k[28], k[29], k[30], k[31]);
    //
    //     // IV is always None
    //     // x.input[14] = 0;
    //     // x.input[15] = 0;
    //
    //     x
    // }

    pub fn get_keystream(&self, pos: u64, inp: &mut [u8]) {
        inp.par_chunks_exact_mut(64)
            .enumerate()
            .for_each(|(i, chunk)| {
                let j12_13 = pos + i as u64;
                let j12 = j12_13 as u32;
                let j13 = (j12_13 >> 32) as u32;

                let j = [
                    self.input[0] as u32,
                    self.input[1] as u32,
                    self.input[2] as u32,
                    self.input[3] as u32,
                    self.input[4] as u32,
                    self.input[5] as u32,
                    self.input[6] as u32,
                    self.input[7] as u32,
                    self.input[8] as u32,
                    self.input[9] as u32,
                    self.input[10] as u32,
                    self.input[11] as u32,
                    j12,
                    j13,
                    self.input[14] as u32,
                    self.input[15] as u32,
                ];

                let mut x = j.clone();

                for _i in 0..4 {
                    quarter_round!(x[0], x[4], x[8], x[12]);
                    quarter_round!(x[1], x[5], x[9], x[13]);
                    quarter_round!(x[2], x[6], x[10], x[14]);
                    quarter_round!(x[3], x[7], x[11], x[15]);
                    quarter_round!(x[0], x[5], x[10], x[15]);
                    quarter_round!(x[1], x[6], x[11], x[12]);
                    quarter_round!(x[2], x[7], x[8], x[13]);
                    quarter_round!(x[3], x[4], x[9], x[14]);
                }

                for i in 0..16 {
                    x[i] = plus(x[i], j[i]);
                }

                u32to8_little(x[0], &mut chunk[0..=3]);
                u32to8_little(x[1], &mut chunk[4..=7]);
                u32to8_little(x[2], &mut chunk[8..=11]);
                u32to8_little(x[3], &mut chunk[12..=15]);
                u32to8_little(x[4], &mut chunk[16..=19]);
                u32to8_little(x[5], &mut chunk[20..=23]);
                u32to8_little(x[6], &mut chunk[24..=27]);
                u32to8_little(x[7], &mut chunk[28..=31]);
                u32to8_little(x[8], &mut chunk[32..=35]);
                u32to8_little(x[9], &mut chunk[36..=39]);
                u32to8_little(x[10], &mut chunk[40..=43]);
                u32to8_little(x[11], &mut chunk[44..=47]);
                u32to8_little(x[12], &mut chunk[48..=51]);
                u32to8_little(x[13], &mut chunk[52..=55]);
                u32to8_little(x[14], &mut chunk[56..=59]);
                u32to8_little(x[15], &mut chunk[60..=63]);
            })
    }
}

