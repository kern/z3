//! Planar conversion kernels over explicit, disjoint source/destination slices.

use std::ops::Range;

use super::{convert_3_to_4, expand_3_to_4_high};

/// Write converted little-endian words directly, retaining the public C
/// adapters' short-destination behavior and leaving any suffix untouched.
pub(super) fn convert_words(destination: &mut [u16], source: &[u8], count: usize, high: bool) {
    let source = &source[..count * 24];
    for (index, word) in destination.iter_mut().take(count * 16).enumerate() {
        let tile = index / 16;
        let row = index % 8;
        let input = tile * 24;
        let p0 = source[input + row * 2];
        let p1 = source[input + row * 2 + 1];
        *word = if index % 16 < 8 {
            u16::from_le_bytes([p0, p1])
        } else {
            let p2 = source[input + 16 + row];
            u16::from_le_bytes([p2, if high { p0 | p1 | p2 } else { 0 }])
        };
    }
}

/// C leaves eight reverse-ordered plane OR masks in low WRAM. Compute only
/// that observable result before releasing a source borrow, not a source copy.
pub(super) fn high_plane_scratch(source: &[u8]) -> [u8; 16] {
    let final_tile = &source[63 * 24..63 * 24 + 16];
    let mut scratch = [0; 16];
    for (index, pair) in final_tile.chunks_exact(2).enumerate() {
        scratch[(7 - index) * 2] = pair[0] | pair[1];
    }
    scratch
}

/// Valid gameplay regions are disjoint and are borrowed without allocation.
/// The compatibility entrypoints also permit overlapping offsets; retain
/// their former snapshot semantics explicitly for that case.
pub(super) fn with_wram_regions<T>(
    wram: &mut [u8],
    source: Range<usize>,
    destination: Range<usize>,
    operation: impl FnOnce(&mut [u8], &[u8]) -> T,
) -> T {
    if source.end <= destination.start {
        let (before, after) = wram.split_at_mut(destination.start);
        operation(&mut after[..destination.len()], &before[source])
    } else if destination.end <= source.start {
        let (before, after) = wram.split_at_mut(source.start);
        operation(&mut before[destination], &after[..source.len()])
    } else {
        let snapshot = wram[source].to_vec();
        operation(&mut wram[destination], &snapshot)
    }
}

pub(super) fn planar_wram(
    wram: &mut [u8],
    destination: usize,
    source: usize,
    count: usize,
    high: bool,
) {
    with_wram_regions(
        wram,
        source..source + count * 24,
        destination..destination + count * 32,
        |destination, source| convert_3_to_4(destination, source, count, high, false),
    );
}

pub(super) fn expand_pair_from_wram(
    wram: &mut [u8],
    destination: usize,
    source: Range<usize>,
    base: usize,
    count: usize,
) {
    with_wram_regions(
        wram,
        source,
        destination..destination + count * 64,
        |destination, source| expand_pair(destination, source, base, count),
    );
}

pub(super) fn expand_pair(destination: &mut [u8], source: &[u8], base: usize, count: usize) {
    expand_3_to_4_high(destination, source, base, count);
    expand_3_to_4_high(
        &mut destination[count * 32..],
        &source[0x180..],
        base + 0x180,
        count,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_word_conversion_matches_byte_planes_for_every_byte_pattern() {
        for high in [false, true] {
            for seed in 0..=u8::MAX {
                for count in [0, 1, 2, 64, 128] {
                    let source = (0..count * 24)
                        .map(|index| (index as u8).wrapping_mul(37).wrapping_add(seed))
                        .collect::<Vec<_>>();
                    let mut expected = vec![0; count * 32];
                    convert_3_to_4(&mut expected, &source, count, high, true);
                    let mut actual = vec![0xa5a5; count * 16 + 3];
                    convert_words(&mut actual, &source, count, high);
                    assert_eq!(
                        actual[..count * 16],
                        expected
                            .chunks_exact(2)
                            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(&actual[count * 16..], &[0xa5a5; 3]);
                }
            }
        }
    }

    #[test]
    fn partial_vram_destinations_keep_the_legacy_prefix_contract() {
        let source = (0..0x600).map(|index| index as u8).collect::<Vec<_>>();
        for high in [false, true] {
            let mut expected = [0; 0x800];
            convert_3_to_4(&mut expected, &source, 64, high, true);
            let expected = expected
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect::<Vec<_>>();
            for length in [0, 1, 7, 8, 9, 0x3ff, 0x400, 0x403] {
                let mut actual = vec![0xdead; length];
                convert_words(&mut actual, &source, 64, high);
                let written = length.min(0x400);
                assert_eq!(actual[..written], expected[..written]);
                assert!(actual[written..].iter().all(|&word| word == 0xdead));
            }
        }
    }

    #[test]
    fn disjoint_regions_borrow_the_original_allocations_in_either_order() {
        let mut wram = [0; 128];
        let base = wram.as_ptr();
        for (source, destination) in [(0..16, 64..96), (64..80, 0..32)] {
            let expected_source = base.wrapping_add(source.start);
            let expected_destination = base.wrapping_add(destination.start);
            with_wram_regions(&mut wram, source, destination, |destination, source| {
                assert_eq!(source.as_ptr(), expected_source);
                assert_eq!(destination.as_ptr(), expected_destination);
            });
        }
    }

    #[test]
    fn planar_wram_preserves_snapshot_behavior_when_regions_overlap() {
        for high in [false, true] {
            for (destination, source) in [(0x20, 0x200), (0x200, 0x20), (0x80, 0x80), (0x80, 0x90)]
            {
                let mut actual = (0..0x800)
                    .map(|index| (index * 37) as u8)
                    .collect::<Vec<_>>();
                let mut expected = actual.clone();
                let snapshot = actual[source..source + 8 * 24].to_vec();
                convert_3_to_4(
                    &mut expected[destination..destination + 8 * 32],
                    &snapshot,
                    8,
                    high,
                    false,
                );
                planar_wram(&mut actual, destination, source, 8, high);
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn paired_expansion_preserves_stride_and_a_single_snapshot_across_both_halves() {
        for destination in [0, 0x1010] {
            let mut actual = (0..0x2400)
                .map(|index| (index * 29) as u8)
                .collect::<Vec<_>>();
            let mut expected = actual.clone();
            let source = actual[0x1018..0x1800].to_vec();
            expand_3_to_4_high(&mut expected[destination..], &source, 0x14018, 8);
            expand_3_to_4_high(
                &mut expected[destination + 8 * 32..],
                &source[0x180..],
                0x14198,
                8,
            );
            expand_pair_from_wram(&mut actual, destination, 0x1018..0x1800, 0x14018, 8);
            assert_eq!(actual, expected);
        }
    }
}
