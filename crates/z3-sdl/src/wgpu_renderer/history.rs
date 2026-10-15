//! Ownership of previous-frame textures and their presentation clock.
//!
//! Recording happens only after a successful present; timeouts, occlusion,
//! and acquisition errors must not advance this ring or its shader counter.

use super::resources::{SourceTexture, ensure_history_texture};

pub(super) struct FrameHistory {
    textures: Vec<Option<SourceTexture>>,
    frame_count: u64,
}

impl FrameHistory {
    pub(super) fn new(slots: usize) -> Self {
        Self {
            textures: (0..slots).map(|_| None).collect(),
            frame_count: 0,
        }
    }

    pub(super) fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub(super) fn bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        original: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let mut history_entries = Vec::with_capacity(self.textures.len() * 2);
        for history_index in 0..self.textures.len() {
            let view = previous_slot(self.frame_count, history_index, self.textures.len())
                .and_then(|slot| self.textures[slot].as_ref())
                .map(|texture| &texture.view)
                .unwrap_or(original);
            history_entries.push(wgpu::BindGroupEntry {
                binding: history_index as u32 * 2,
                resource: wgpu::BindingResource::TextureView(view),
            });
            history_entries.push(wgpu::BindGroupEntry {
                binding: history_index as u32 * 2 + 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("z3 wgpu previous-frame bind group"),
            layout,
            entries: &history_entries,
        })
    }

    pub(super) fn record_presented(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        pixels: &[u8],
    ) {
        if !self.textures.is_empty() {
            let slot = self.frame_count as usize % self.textures.len();
            ensure_history_texture(device, &mut self.textures[slot], size.0, size.1);
            let history = self.textures[slot].as_ref().expect("history was created");
            queue.write_texture(
                history.texture.as_image_copy(),
                pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.0 * 4),
                    rows_per_image: Some(size.1),
                },
                wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.advance();
    }

    fn advance(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

fn previous_slot(frame_count: u64, history_index: usize, slots: usize) -> Option<usize> {
    if slots == 0 || history_index >= slots {
        return None;
    }
    frame_count
        .checked_sub(1 + history_index as u64)
        .map(|frame| frame as usize % slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_slots_follow_presented_frames_through_ring_wraps() {
        for slots in [1, 2, 7] {
            let mut history = FrameHistory::new(slots);
            let mut written = vec![None; slots];
            for frame in 0..64u64 {
                assert_eq!(history.frame_count(), frame);
                for index in 0..slots {
                    let actual = previous_slot(history.frame_count(), index, slots)
                        .and_then(|slot| written[slot]);
                    assert_eq!(actual, frame.checked_sub(1 + index as u64));
                }
                written[frame as usize % slots] = Some(frame);
                history.advance();
            }
        }
    }

    #[test]
    fn shader_clock_advances_even_without_previous_frame_textures() {
        let mut history = FrameHistory::new(0);
        assert!(history.textures.is_empty());
        assert_eq!(history.frame_count(), 0);
        assert_eq!(previous_slot(0, 0, 0), None);
        history.advance();
        assert_eq!(history.frame_count(), 1);
    }

    #[test]
    fn counter_wrap_restores_the_original_frame_fallback_policy() {
        let mut history = FrameHistory::new(7);
        history.frame_count = u64::MAX;
        assert!(previous_slot(history.frame_count(), 0, 7).is_some());
        history.advance();
        assert_eq!(history.frame_count(), 0);
        for index in 0..7 {
            assert_eq!(previous_slot(history.frame_count(), index, 7), None);
        }
        history.advance();
        assert_eq!(previous_slot(history.frame_count(), 0, 7), Some(0));
        assert_eq!(previous_slot(history.frame_count(), 1, 7), None);
    }
}
