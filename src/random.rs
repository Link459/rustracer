//use rand::{rand_core, Rng, TryRng};

pub struct RngState<S> {
    state: S,
    inc: S,
}

pub fn pcg32_random(rng: &mut RngState<u64>) -> u32 {
    let old_state = rng.state;

    rng.state = old_state * 6364136223846793005 + (rng.inc | 1);

    let xor_shifted = ((old_state >> 18) ^ old_state) >> 27;
    let rot = old_state >> 59;
    return ((xor_shifted >> rot) | (xor_shifted << ((-(rot as i64)) & 31))) as u32;
}

pub fn pcg64_random(rng: &mut RngState<u128>) -> u64 {
    let old_state = rng.state;

    rng.state = old_state * 6364136223846793005 + (rng.inc | 1);

    let xor_shifted = ((old_state >> 18) ^ old_state) >> 27;
    let rot = old_state >> 59;
    return ((xor_shifted >> rot) | (xor_shifted << ((-(rot as i128)) & 31))) as u64;
}

/*impl Rng for RngState<u128> {
    fn next_u32(&mut self) -> u32 {
        return (self.next_u64() >> 32) as u32;
    }

    fn next_u64(&mut self) -> u64 {
        return pcg64_random(self);
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        rand_core::utils::fill_bytes_via_next_word(dst, || Result::Ok(self.next_u64()));
    }
}*/
