//! Typed runtime views over the Python-generated `zelda3_assets.dat` bundle.
//!
//! The C runtime casts selected byte payloads to little-endian word arrays.
//! Rust keeps the file parser zero-copy, then decodes those word arrays once
//! here so gameplay code can use safe, naturally aligned slices.

use z3_core::assets::{AssetError, AssetFile, packed_array_entry};

use crate::dungeon::DungeonEntrance;
use crate::load_gfx::GfxAssets;
use crate::messaging::MessagingAssets;
use crate::overworld::OverworldAssets;

pub struct RuntimeAssets<'a> {
    file: AssetFile<'a>,
    words: Vec<Vec<u16>>,
    signed_bytes: Vec<Vec<i8>>,
    signed_words: Vec<Vec<i16>>,
    gloves_color: [u16; 2],
}

impl<'a> RuntimeAssets<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AssetError> {
        Self::parse_with_gloves(bytes, crate::load_gfx::GLOVES_COLOR)
    }

    pub fn parse_with_gloves(bytes: &'a [u8], gloves_color: [u16; 2]) -> Result<Self, AssetError> {
        let file = AssetFile::parse(bytes)?;
        let words = file
            .iter()
            .map(|asset| {
                asset
                    .chunks_exact(2)
                    .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                    .collect()
            })
            .collect();
        let signed_bytes = file
            .iter()
            .map(|asset| asset.iter().map(|&byte| byte as i8).collect())
            .collect();
        let signed_words = file
            .iter()
            .map(|asset| {
                asset
                    .chunks_exact(2)
                    .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
                    .collect()
            })
            .collect();
        Ok(Self {
            file,
            words,
            signed_bytes,
            signed_words,
            gloves_color,
        })
    }

    #[must_use]
    pub fn file(&self) -> &AssetFile<'a> {
        &self.file
    }

    pub fn bytes(&self, index: usize) -> &[u8] {
        self.file.get(index).expect("runtime asset index")
    }

    pub fn words(&self, index: usize) -> &[u16] {
        &self.words[index]
    }

    pub fn signed_bytes(&self, index: usize) -> &[i8] {
        &self.signed_bytes[index]
    }

    pub fn signed_words(&self, index: usize) -> &[i16] {
        &self.signed_words[index]
    }

    pub fn packed(&self, asset: usize, index: usize) -> &[u8] {
        packed_array_entry(self.bytes(asset), index).expect("packed runtime asset entry")
    }

    #[must_use]
    pub fn language(&self, requested: Option<&str>) -> LanguageAssets<'_> {
        let mut configuration = [0u8; 3];
        if let Some(requested) = requested {
            for mapping in (0..).map_while(|index| packed_array_entry(self.bytes(96), index)) {
                let Some(name) = packed_array_entry(mapping, 0) else {
                    continue;
                };
                if name == requested.as_bytes()
                    && let Some(found) = packed_array_entry(mapping, 1)
                    && found.len() >= 3
                {
                    configuration.copy_from_slice(&found[..3]);
                    break;
                }
            }
        }
        LanguageAssets {
            dialogue: self.packed(94, usize::from(configuration[0])),
            fonts: self.packed(95, usize::from(configuration[1])),
            flags: configuration[2],
        }
    }

    #[must_use]
    pub fn messaging(&self, requested: Option<&str>) -> MessagingAssetSet<'_> {
        let language = self.language(requested);
        let dictionary_block = packed_array_entry(language.dialogue, 0)
            .expect("dialogue asset contains dictionary block");
        let message_block = packed_array_entry(language.dialogue, 1)
            .expect("dialogue asset contains message block");
        let font_tiles =
            packed_array_entry(language.fonts, 0).expect("dialogue font asset contains tiles");
        let font_widths =
            packed_array_entry(language.fonts, 1).expect("dialogue font asset contains widths");
        MessagingAssetSet {
            dialogue_flags: language.flags,
            dictionary: packed_entries(dictionary_block),
            messages: packed_entries(message_block),
            font_tiles,
            font_widths,
            light_overworld_tilemap: self.bytes(67),
            dungeon_map_floor_layout: self.bytes(97),
            dungeon_map_info: self.bytes(98),
        }
    }

    #[must_use]
    pub fn nmi_assets(&self) -> crate::nmi::NmiAssets<'_> {
        crate::nmi::NmiAssets {
            link_graphics: self.bytes(57),
            bg_tilemaps: [
                self.bytes(99),
                self.bytes(100),
                self.bytes(101),
                self.bytes(102),
                self.bytes(103),
                self.bytes(104),
            ],
            light_overworld_tilemap: self.bytes(67),
        }
    }

    #[must_use]
    pub fn dungeon_object_data(&self, word_offset: usize) -> &[u16] {
        &self.words(69)[word_offset..]
    }

    #[must_use]
    pub fn dungeon_room_door_info(&self, room: usize) -> &[u16] {
        let byte_offset = usize::from(self.words(5)[room]);
        &self.words(3)[byte_offset / 2..]
    }

    #[must_use]
    pub fn dungeon_room_header(&self, room: usize) -> &[u8] {
        &self.bytes(6)[usize::from(self.words(7)[room])..]
    }

    #[must_use]
    pub fn dungeon_room_layout(&self, room: usize) -> &[u8] {
        &self.bytes(3)[usize::from(self.words(4)[room])..]
    }

    #[must_use]
    pub fn default_room_layout(&self, layout: usize) -> &[u8] {
        &self.bytes(46)[usize::from(self.words(47)[layout])..]
    }

    #[must_use]
    pub fn dungeon_room_chests(&self) -> &[u8] {
        self.bytes(8)
    }

    #[must_use]
    pub fn dungeon_secrets(&self) -> &[u8] {
        self.bytes(50)
    }

    #[must_use]
    pub fn dungeon_movable_block(&self, index: usize) -> (u16, u16) {
        let data = self.words(53);
        (data[index * 2], data[index * 2 + 1])
    }

    #[must_use]
    pub fn dungeon_torch_data(&self, index: usize) -> u16 {
        self.words(54)[index]
    }

    #[must_use]
    pub fn dungeon_entrance(&self, starting_point: bool, index: usize) -> DungeonEntrance {
        let (
            room,
            relative,
            scroll_x,
            scroll_y,
            player_x,
            player_y,
            camera_x,
            camera_y,
            blockset,
            floor,
            palace,
            doorway_orientation,
            starting_bg,
            quadrant1,
            quadrant2,
            door_settings,
            entrance,
            music,
        ) = if starting_point {
            (
                self.words(28)[index],
                self.bytes(29),
                self.words(30)[index],
                self.words(31)[index],
                self.words(32)[index],
                self.words(33)[index],
                self.words(34)[index],
                self.words(35)[index],
                self.bytes(36)[index],
                self.bytes(37)[index],
                self.bytes(38)[index],
                self.bytes(39)[index],
                self.bytes(40)[index],
                self.bytes(41)[index],
                self.bytes(42)[index],
                self.words(43)[index],
                u16::from(self.bytes(44)[index]),
                self.bytes(45)[index],
            )
        } else {
            (
                self.words(11)[index],
                self.bytes(12),
                self.words(13)[index],
                self.words(14)[index],
                self.words(15)[index],
                self.words(16)[index],
                self.words(17)[index],
                self.words(18)[index],
                self.bytes(19)[index],
                self.bytes(20)[index],
                self.bytes(21)[index],
                self.bytes(22)[index],
                self.bytes(23)[index],
                self.bytes(24)[index],
                self.bytes(25)[index],
                self.words(26)[index],
                index as u16,
                self.bytes(27)[index],
            )
        };
        let mut relative_coords = [0; 8];
        relative_coords.copy_from_slice(&relative[index * 8..index * 8 + 8]);
        DungeonEntrance {
            entrance,
            room,
            scroll_y,
            scroll_x,
            player_y,
            player_x,
            camera_y,
            camera_x,
            door_settings,
            relative_coords,
            blockset,
            floor,
            palace,
            doorway_orientation,
            starting_bg,
            quadrant1,
            quadrant2,
            music,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LanguageAssets<'a> {
    pub dialogue: &'a [u8],
    pub fonts: &'a [u8],
    pub flags: u8,
}

/// Owned slice table for the nested packed arrays used by the messaging
/// runtime. The payload bytes still borrow directly from the validated asset
/// bundle; only the pointer tables are materialized.
pub struct MessagingAssetSet<'a> {
    dialogue_flags: u8,
    dictionary: Vec<&'a [u8]>,
    messages: Vec<&'a [u8]>,
    font_tiles: &'a [u8],
    font_widths: &'a [u8],
    light_overworld_tilemap: &'a [u8],
    dungeon_map_floor_layout: &'a [u8],
    dungeon_map_info: &'a [u8],
}

impl MessagingAssetSet<'_> {
    #[must_use]
    pub fn as_assets(&self) -> MessagingAssets<'_> {
        MessagingAssets {
            dialogue_flags: self.dialogue_flags,
            dictionary: &self.dictionary,
            messages: &self.messages,
            font_tiles: self.font_tiles,
            font_widths: self.font_widths,
            light_overworld_tilemap: self.light_overworld_tilemap,
            dungeon_map_floor_layout: self.dungeon_map_floor_layout,
            dungeon_map_info: self.dungeon_map_info,
        }
    }
}

fn packed_entries(data: &[u8]) -> Vec<&[u8]> {
    (0..)
        .map_while(|index| packed_array_entry(data, index))
        .collect()
}

impl GfxAssets for RuntimeAssets<'_> {
    fn sprite(&self, index: usize) -> &[u8] {
        packed_array_entry(self.bytes(64), index).expect("sprite graphics pack")
    }

    fn background(&self, index: usize) -> &[u8] {
        packed_array_entry(self.bytes(65), index).expect("background graphics pack")
    }

    fn raw(&self, index: usize) -> &[u8] {
        self.bytes(index)
    }

    fn dialogue_font(&self) -> &[u8] {
        let language = packed_array_entry(self.bytes(95), 0).expect("dialogue font language");
        packed_array_entry(language, 0).expect("dialogue font graphics")
    }

    fn gloves_color(&self, index: usize) -> u16 {
        self.gloves_color[index]
    }
}

impl OverworldAssets for RuntimeAssets<'_> {
    fn entrance_areas(&self) -> &[u16] {
        self.words(124)
    }
    fn entrance_positions(&self) -> &[u16] {
        self.words(125)
    }
    fn sign_text(&self) -> &[u16] {
        self.words(110)
    }
    fn background_palettes(&self) -> &[u8] {
        self.bytes(109)
    }
    fn sprite_graphics(&self) -> &[u8] {
        self.bytes(161)
    }
    fn sprite_palettes(&self) -> &[u8] {
        self.bytes(162)
    }
    fn sprite_offsets(&self) -> &[u16] {
        self.words(159)
    }
    fn sprites(&self) -> &[u8] {
        self.bytes(160)
    }
    fn map16_to_map8(&self) -> &[u16] {
        self.words(70)
    }
    fn map8_tile_attributes(&self) -> &[u8] {
        self.bytes(163)
    }
    fn enemy_damage_data(&self) -> &[u8] {
        self.bytes(56)
    }
    fn overworld_music_sets(&self) -> &[u8] {
        self.bytes(111)
    }
    fn overworld_music_sets2(&self) -> &[u8] {
        self.bytes(112)
    }
    fn fall_hole_areas(&self) -> &[u16] {
        self.words(127)
    }
    fn fall_hole_positions(&self) -> &[u16] {
        self.words(128)
    }
    fn fall_hole_entrances(&self) -> &[u8] {
        self.bytes(129)
    }
    fn whirlpool_areas(&self) -> &[u16] {
        self.words(123)
    }
    fn map32_to_map16(&self) -> [&[u8]; 4] {
        [
            self.bytes(60),
            self.bytes(61),
            self.bytes(62),
            self.bytes(63),
        ]
    }
    fn overworld_hibytes(&self, screen: usize) -> &[u8] {
        packed_array_entry(self.bytes(105), screen).expect("overworld high-byte map")
    }
    fn overworld_lobytes(&self, screen: usize) -> &[u8] {
        packed_array_entry(self.bytes(106), screen).expect("overworld low-byte map")
    }
    fn overworld_map_is_small(&self) -> &[u8] {
        self.bytes(107)
    }
    fn overworld_secrets(&self) -> &[u8] {
        self.bytes(158)
    }
    fn overworld_secret_offsets(&self) -> &[u16] {
        self.words(157)
    }
    fn exit_screen_indices(&self) -> &[u8] {
        self.bytes(130)
    }
    fn exit_rooms(&self) -> &[u16] {
        self.words(131)
    }
    fn exit_map16_source_offsets(&self) -> &[u16] {
        self.words(132)
    }
    fn exit_scroll_x(&self) -> &[u16] {
        self.words(133)
    }
    fn exit_scroll_y(&self) -> &[u16] {
        self.words(134)
    }
    fn exit_link_x(&self) -> &[u16] {
        self.words(135)
    }
    fn exit_link_y(&self) -> &[u16] {
        self.words(136)
    }
    fn exit_camera_x(&self) -> &[u16] {
        self.words(137)
    }
    fn exit_camera_y(&self) -> &[u16] {
        self.words(138)
    }
    fn exit_normal_doors(&self) -> &[u16] {
        self.words(139)
    }
    fn exit_fancy_doors(&self) -> &[u16] {
        self.words(140)
    }
    fn exit_unk1(&self) -> &[i8] {
        self.signed_bytes(141)
    }
    fn exit_unk3(&self) -> &[i8] {
        self.signed_bytes(142)
    }
    fn bird_scroll_x(&self) -> &[u16] {
        self.words(115)
    }
    fn bird_scroll_y(&self) -> &[u16] {
        self.words(116)
    }
    fn bird_link_x(&self) -> &[u16] {
        self.words(117)
    }
    fn bird_link_y(&self) -> &[u16] {
        self.words(118)
    }
    fn bird_camera_x(&self) -> &[u16] {
        self.words(119)
    }
    fn bird_camera_y(&self) -> &[u16] {
        self.words(120)
    }
    fn bird_map16_source_offsets(&self) -> &[u16] {
        self.words(114)
    }
    fn bird_screen_indices(&self) -> &[u8] {
        self.bytes(113)
    }
    fn bird_unk1(&self) -> &[i8] {
        self.signed_bytes(121)
    }
    fn bird_unk3(&self) -> &[i8] {
        self.signed_bytes(122)
    }
    fn overworld_aux_tile_themes(&self) -> &[u8] {
        self.bytes(108)
    }
    fn entrance_ids(&self) -> &[u8] {
        self.bytes(126)
    }
    fn special_exit_top(&self) -> &[u16] {
        self.words(143)
    }
    fn special_exit_bottom(&self) -> &[u16] {
        self.words(144)
    }
    fn special_exit_left(&self) -> &[u16] {
        self.words(145)
    }
    fn special_exit_right(&self) -> &[u16] {
        self.words(146)
    }
    fn special_exit_up_target(&self) -> &[i16] {
        self.signed_words(147)
    }
    fn special_exit_down_target(&self) -> &[i16] {
        self.signed_words(148)
    }
    fn special_exit_left_target(&self) -> &[i16] {
        self.signed_words(149)
    }
    fn special_exit_right_target(&self) -> &[i16] {
        self.signed_words(150)
    }
    fn special_exit_left_edge(&self) -> &[u16] {
        self.words(151)
    }
    fn special_exit_direction(&self) -> &[u8] {
        self.bytes(152)
    }
    fn special_exit_sprite_gfx(&self) -> &[u8] {
        self.bytes(153)
    }
    fn special_exit_aux_gfx(&self) -> &[u8] {
        self.bytes(154)
    }
    fn special_exit_bg_palette(&self) -> &[u8] {
        self.bytes(155)
    }
    fn special_exit_sprite_palette(&self) -> &[u8] {
        self.bytes(156)
    }
}
