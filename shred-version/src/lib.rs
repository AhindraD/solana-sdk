//! Calculation of [shred] versions.
//!
//! [shred]: https://solana.com/docs/terminology#shred

use {solana_hard_forks::HardForks, solana_hash::Hash, solana_sha256_hasher::hashv};

pub fn version_from_hash(hash: &Hash) -> u16 {
    let hash = hash.as_ref();
    let mut accum = [0u8; 2];
    hash.chunks(2).for_each(|seed| {
        accum
            .iter_mut()
            .zip(seed)
            .for_each(|(accum, seed)| *accum ^= *seed)
    });
    // convert accum into a u16
    // Because accum[0] is a u8, 8bit left shift of the u16 can never overflow
    #[allow(clippy::arithmetic_side_effects)]
    let version = ((accum[0] as u16) << 8) | accum[1] as u16;

    // ensure version is never zero, to avoid looking like an uninitialized version
    version.saturating_add(1)
}

pub fn compute_shred_version(genesis_hash: &Hash, hard_forks: Option<&HardForks>) -> u16 {
    let mut hash = *genesis_hash;
    if let Some(hard_forks) = hard_forks {
        for &(slot, count) in hard_forks.iter() {
            let buf = [slot.to_le_bytes(), (count as u64).to_le_bytes()].concat();
            hash = hashv(&[hash.as_ref(), &buf]);
        }
    }

    version_from_hash(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_shred_version() {
        assert_eq!(compute_shred_version(&Hash::default(), None), 1);
        let mut hard_forks = HardForks::default();
        assert_eq!(
            compute_shred_version(&Hash::default(), Some(&hard_forks)),
            1
        );
        hard_forks.register(1);
        assert_eq!(
            compute_shred_version(&Hash::default(), Some(&hard_forks)),
            55551
        );
        hard_forks.register(1);
        assert_eq!(
            compute_shred_version(&Hash::default(), Some(&hard_forks)),
            46353
        );
    }
}
