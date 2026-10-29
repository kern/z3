//! Frozen methods copied verbatim from player.rs at 2dc8f79.
//! Only the enclosing type/entry adapter is new; no migrated field APIs.
use crate::ram::Ram;

const SWIMMING_TAB2: [u8; 2] = [32, 8];

struct FrozenSwim<'a, 'r> {
    ram: &'a mut Ram<'r>,
}

impl FrozenSwim<'_, '_> {
    pub fn link_flag_max_accels(&mut self) {
        if self.ram.read_u8(0x34a) == 0 {
            return;
        }
        for i in 0..2 {
            let acceleration = self.ram.read_u16(0x33c + i * 2);
            if acceleration != 0 {
                self.ram.write_u16(0x334 + i * 2, acceleration);
                self.ram.write_u16(0x32b + i * 2, 1)
            }
        }
    }

    pub fn link_set_ice_max_accel(&mut self) {
        if self.ram.read_u8(0x34a) != 0 {
            self.ram.write_u16(0x334, 0x180);
            self.ram.write_u16(0x336, 0x180)
        }
    }

    pub fn link_set_momentum(&mut self) {
        let joy = self.ram.read_u8(0xf0) & 0xf;
        let mut mask = 12;
        let mut bit = 8;
        for i in 0..2 {
            if joy & mask != 0 {
                let moving = self.ram.read_u8(0x34a);
                self.ram.write_u16(
                    0x326 + i * 2,
                    u16::from(if moving != 0 {
                        SWIMMING_TAB2[usize::from(moving - 1)]
                    } else {
                        32
                    }),
                );
                if (self.ram.read_u8(0x340) | self.ram.read_u8(0x67)) & mask == mask {
                    self.ram.write_u16(0x32b + i * 2, 2)
                } else {
                    self.ram.write_u16(0x338 + i * 2, u16::from(joy & bit == 0));
                    self.ram.write_u16(0x32b + i * 2, 0)
                }
                if self.ram.read_u16(0x334 + i * 2) == 0 {
                    self.ram.write_u16(0x334 + i * 2, 240)
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub fn link_set_the_max_accel(&mut self) {
        if self.ram.read_u8(0x34a) != 0 || self.ram.read_u8(0x34f) != 0 {
            return;
        }
        let mut mask = 12;
        for i in 0..2 {
            let var1_addr = 0x32f + i * 2;
            let var5_addr = 0x32b + i * 2;
            let var7_addr = 0x33c + i * 2;
            let var9_addr = 0x334 + i * 2;
            if self.ram.read_u8(0xf0) & mask != 0 && self.ram.read_u16(var5_addr) != 2 {
                let velocity = self.ram.read_u16(var7_addr);
                if self.ram.read_u16(var1_addr) != 0
                    || (velocity >= 240 && velocity >= self.ram.read_u16(var9_addr))
                {
                    self.ram.write_u16(var5_addr, 0);
                    if velocity >= 240 {
                        self.ram.write_u16(var1_addr, 1);
                        self.ram.write_u16(var5_addr, 1)
                    } else {
                        self.ram.write_u16(var9_addr, 240);
                        self.ram.write_u16(var1_addr, 0)
                    }
                }
            } else {
                self.ram.write_u16(var9_addr, 240);
                self.ram.write_u16(var1_addr, 0)
            }
            mask >>= 2;
        }
    }

    pub fn link_handle_swim_accels(&mut self) {
        const ACCELERATION_STEPS: [u16; 9] = [128, 160, 192, 224, 256, 288, 320, 352, 384];
        let mut mask = 12;
        for i in 0..2 {
            if self.ram.read_u8(0xf0) & mask != 0 {
                let velocity = self.ram.read_u16(0x33c + i * 2);
                let maximum = self.ram.read_u16(0x334 + i * 2);
                if velocity != 0 && maximum >= 384 {
                    let mut step = ACCELERATION_STEPS[8];
                    for candidate in ACCELERATION_STEPS {
                        step = candidate;
                        if candidate >= velocity {
                            break;
                        }
                    }
                    self.ram.write_u16(0x334 + i * 2, step)
                } else if maximum != 0 {
                    self.ram
                        .write_u16(0x334 + i * 2, maximum.wrapping_add(160).min(384))
                } else {
                    self.ram.write_u16(0x33c + i * 2, 1);
                    self.ram.write_u16(0x334 + i * 2, 240)
                }
            }
            mask >>= 2;
        }
    }

    pub fn handle_swim_stroke_and_subpixels(&mut self) {
        const SWIMMING_ACCELERATION: [i8; 12] = [8, -12, -8, -16, 4, -6, -12, -6, 10, -16, -12, -6];
        const DIRECTION_MASK: [u8; 2] = [!0xc, !3];
        const DIRECTION_BIT: [u8; 4] = [8, 4, 2, 1];

        self.ram.set_player_actual_velocity_x(0);
        self.ram.set_player_actual_velocity_y(0);
        let mut velocity = [0u16; 2];
        for i in (0..2).rev() {
            let var3_addr = 0x326 + i * 2;
            let var5_addr = 0x32b + i * 2;
            let var9_addr = 0x334 + i * 2;
            let var11_addr = 0x338 + i * 2;
            let var7_addr = 0x33c + i * 2;
            let countdown = self.ram.read_u16(var3_addr).wrapping_sub(1);
            self.ram.write_u16(var3_addr, countdown);
            if (countdown as i16) < 0 {
                self.ram.write_u16(var3_addr, 0);
                self.ram.write_u16(var5_addr, 1)
            }
            let mut table_index = self.ram.read_u16(var5_addr);
            if self.ram.read_u8(0x34a) != 0 {
                table_index =
                    table_index.wrapping_add(u16::from(self.ram.read_u8(0x34a)).wrapping_mul(4))
            }
            let mut sum = self
                .ram
                .read_u16(var7_addr)
                .wrapping_add(SWIMMING_ACCELERATION[table_index as usize] as i16 as u16);
            if (sum as i16) <= 0 {
                let direction = self.ram.read_u8(0x67) & DIRECTION_MASK[i];
                self.ram.write_u8(0x67, direction);
                self.ram.write_u8(0x26, direction);
                if self.ram.read_u16(var5_addr) == 2 {
                    self.ram.write_u16(var5_addr, 0);
                    self.ram.write_u16(var9_addr, 240);
                    self.ram.write_u16(var7_addr, 2)
                } else {
                    self.ram.write_u16(var5_addr, 0);
                    self.ram.write_u16(var9_addr, 0);
                    self.ram.write_u16(var7_addr, 0)
                }
            } else {
                let direction_index = self.ram.read_u16(var11_addr) as usize + i * 2;
                self.ram.write_u8(
                    0x67,
                    self.ram.read_u8(0x67) | DIRECTION_BIT[direction_index],
                );
                if sum >= self.ram.read_u16(var9_addr) {
                    sum = self.ram.read_u16(var9_addr)
                }
                self.ram.write_u16(var7_addr, sum)
            }
            velocity[i] = self.ram.read_u16(var7_addr);
            if self.ram.read_u8(0x6a) | self.ram.read_u8(0x6b) != 0 {
                velocity[i] = velocity[i].wrapping_sub(velocity[i] >> 2)
            }
            if self.ram.read_u16(var11_addr) == 0 {
                velocity[i] = 0u16.wrapping_sub(velocity[i])
            }
        }
        self.player_something_with_velocity_tired_or_swim(velocity[1], velocity[0])
    }

    pub fn link_reset_swimming_state(&mut self) {
        self.ram.write_u8(0x2cb, 0);
        self.ram.write_u8(0x34f, 0);
        self.ram.write_u8(0x32a, 0);
        for address in [
            0x32f, 0x331, 0x326, 0x328, 0x32b, 0x32d, 0x33c, 0x33e, 0x334, 0x336,
        ] {
            self.ram.write_u16(address, 0)
        }
    }

    fn reset_all_acceleration(&mut self) {
        for address in [
            0x32f, 0x331, 0x326, 0x328, 0x32b, 0x32d, 0x33c, 0x33e, 0x334, 0x336,
        ] {
            self.ram.write_u16(address, 0)
        }
    }

    pub fn player_something_with_velocity_tired_or_swim(
        &mut self,
        x_velocity: u16,
        y_velocity: u16,
    ) {
        let old_x = self.ram.player().x();
        let old_y = self.ram.player().y();
        self.save_safe_return_coordinates();

        let x_fixed = i32::from(old_x) * 256
            + i32::from(self.ram.player().subpixel_x())
            + i32::from(x_velocity as i16);
        self.ram.set_player_subpixel_x(x_fixed as u8);
        self.ram.set_player_x(((x_fixed as u32) >> 8) as u16);
        let x_high = (x_velocity >> 8) as u8;
        self.ram.set_player_actual_velocity_x(
            (if (x_high as i8) < 0 {
                0u8.wrapping_sub(x_high)
            } else {
                x_high
            })
            .wrapping_shl(4)
                | (x_velocity as u8 >> 4),
        );

        let y_fixed = i32::from(old_y) * 256
            + i32::from(self.ram.player().subpixel_y())
            + i32::from(y_velocity as i16);
        self.ram.set_player_subpixel_y(y_fixed as u8);
        self.ram.set_player_y(((y_fixed as u32) >> 8) as u16);
        let y_high = (y_velocity >> 8) as u8;
        self.ram.set_player_actual_velocity_y(
            (if (y_high as i8) < 0 {
                0u8.wrapping_sub(y_high)
            } else {
                y_high
            })
            .wrapping_shl(4)
                | (y_velocity as u8 >> 4),
        );

        if self.ram.read_u8(0x46c) == 4 {
            self.link_apply_moving_floor_velocity()
        }
        self.ram.write_u8(0x69, 0);
        self.ram.write_u8(0x68, 0);
        self.link_handle_velocity_and_sand_drag(old_x, old_y)
    }

    fn save_safe_return_coordinates(&mut self) {
        let y = self.ram.player().y();
        let x = self.ram.player().x();
        self.ram.write_u8(0x3e, y as u8);
        self.ram.write_u8(0x40, (y >> 8) as u8);
        self.ram.write_u8(0x3f, x as u8);
        self.ram.write_u8(0x41, (x >> 8) as u8)
    }

    pub fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        self.ram
            .set_player_y(self.ram.player().y().wrapping_add(self.ram.read_u16(0xb7e)));
        self.ram
            .set_player_x(self.ram.player().x().wrapping_add(self.ram.read_u16(0xb7c)));
        self.ram
            .set_player_velocity_y(self.ram.player().y().wrapping_sub(old_y) as u8);
        self.ram
            .set_player_velocity_x(self.ram.player().x().wrapping_sub(old_x) as u8)
    }

    pub fn link_apply_moving_floor_velocity(&mut self) {
        self.ram.write_u8(0x6a, 0);
        self.ram
            .set_player_y(self.ram.player().y().wrapping_add(self.ram.read_u16(0x310)));
        self.ram
            .set_player_x(self.ram.player().x().wrapping_add(self.ram.read_u16(0x312)))
    }
}

pub(super) fn execute(operation: u8, ram: &mut Ram<'_>) {
    let mut frozen = FrozenSwim { ram };
    match operation {
        0 => frozen.reset_all_acceleration(),
        1 => frozen.link_reset_swimming_state(),
        2 => frozen.link_flag_max_accels(),
        3 => frozen.link_set_ice_max_accel(),
        4 => frozen.link_set_momentum(),
        5 => frozen.link_set_the_max_accel(),
        6 => frozen.link_handle_swim_accels(),
        7 => frozen.handle_swim_stroke_and_subpixels(),
        _ => unreachable!(),
    }
}
