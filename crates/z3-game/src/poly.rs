//! Polyhedral renderer port from `src/poly.c`.

use crate::ram::Ram;

const POLY_CONFIG1: usize = 0x1f02;
const POLY_WHICH_MODEL: usize = 0x1f03;
const POLY_A: usize = 0x1f04;
const POLY_B: usize = 0x1f05;
const POLY_BASE_X: usize = 0x1f06;
const POLY_BASE_Y: usize = 0x1f07;
const POLY_VAR1: usize = 0x1f08;
const POLY_CONFIG_NUM_VERTEX: usize = 0x1f3f;
const POLY_CONFIG_NUM_POLYS: usize = 0x1f40;
const POLY_FROMLUT_PTR2: usize = 0x1f41;
const POLY_FROMLUT_PTR4: usize = 0x1f43;
const POLY_FROMLUT_Z: usize = 0x1f45;
const POLY_FROMLUT_Y: usize = 0x1f46;
const POLY_FROMLUT_X: usize = 0x1f47;
const POLY_F0: usize = 0x1f48;
const POLY_F1: usize = 0x1f4a;
const POLY_F2: usize = 0x1f4c;
const POLY_NUM_VERTEX_IN_POLY: usize = 0x1f4e;
const POLY_RASTER_COLOR_CONFIG: usize = 0x1f4f;
const POLY_SIN_A: usize = 0x1f50;
const POLY_COS_A: usize = 0x1f52;
const POLY_SIN_B: usize = 0x1f54;
const POLY_COS_B: usize = 0x1f56;
const POLY_E0: usize = 0x1f58;
const POLY_E2: usize = 0x1f5a;
const POLY_E3: usize = 0x1f5c;
const POLY_E1: usize = 0x1f5e;
const POLY_ARR_X: usize = 0x1f60;
const POLY_ARR_Y: usize = 0x1f88;
const POLY_TMP0: usize = 0x1fb0;
const POLY_TMP1: usize = 0x1fb2;
const POLY_RASTER_COLOR0: usize = 0x1fb5;
const POLY_RASTER_COLOR1: usize = 0x1fb7;
const POLY_RASTER_DST_PTR: usize = 0x1fb9;
const POLY_TMP2: usize = 0x1fbc;
const POLY_XY_COORDS: usize = 0x1fc0;
const POLY_TOTAL_NUM_STEPS: usize = 0x1fe0;
const POLY_X0_CUR: usize = 0x1fe1;
const POLY_Y0_CUR: usize = 0x1fe2;
const POLY_X0_TARGET: usize = 0x1fe3;
const POLY_Y0_TRIG: usize = 0x1fe4;
const POLY_X0_FRAC: usize = 0x1fe5;
const POLY_X0_STEP: usize = 0x1fe7;
const POLY_CUR_VERTEX_IDX0: usize = 0x1fe9;
const POLY_X1_CUR: usize = 0x1fea;
const POLY_Y1_CUR: usize = 0x1feb;
const POLY_X1_TARGET: usize = 0x1fec;
const POLY_Y1_TRIG: usize = 0x1fed;
const POLY_X1_FRAC: usize = 0x1fee;
const POLY_X1_STEP: usize = 0x1ff0;
const POLY_CUR_VERTEX_IDX1: usize = 0x1ff2;
const POLY_RASTER_NUMFULL: usize = 0x1ffa;
const POLYHEDRAL_BUFFER: usize = 0xe800;

const POLY_SIN_COS: [i8; 320] = [
    0, 2, 3, 5, 6, 8, 9, 11, 12, 14, 16, 17, 19, 20, 22, 23, 24, 26, 27, 29, 30, 32, 33, 34, 36,
    37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 56, 57, 58, 59, 59,
    60, 60, 61, 61, 62, 62, 62, 63, 63, 63, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 63, 63, 63,
    62, 62, 62, 61, 61, 60, 60, 59, 59, 58, 57, 56, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45,
    44, 43, 42, 41, 39, 38, 37, 36, 34, 33, 32, 30, 29, 27, 26, 24, 23, 22, 20, 19, 17, 16, 14, 12,
    11, 9, 8, 6, 5, 3, 2, 0, -2, -3, -5, -6, -8, -9, -11, -12, -14, -16, -17, -19, -20, -22, -23,
    -24, -26, -27, -29, -30, -32, -33, -34, -36, -37, -38, -39, -41, -42, -43, -44, -45, -46, -47,
    -48, -49, -50, -51, -52, -53, -54, -55, -56, -56, -57, -58, -59, -59, -60, -60, -61, -61, -62,
    -62, -62, -63, -63, -63, -64, -64, -64, -64, -64, -64, -64, -64, -64, -64, -64, -63, -63, -63,
    -62, -62, -62, -61, -61, -60, -60, -59, -59, -58, -57, -56, -56, -55, -54, -53, -52, -51, -50,
    -49, -48, -47, -46, -45, -44, -43, -42, -41, -39, -38, -37, -36, -34, -33, -32, -30, -29, -27,
    -26, -24, -23, -22, -20, -19, -17, -16, -14, -12, -11, -9, -8, -6, -5, -3, -2, 0, 2, 3, 5, 6,
    8, 9, 11, 12, 14, 16, 17, 19, 20, 22, 23, 24, 26, 27, 29, 30, 32, 33, 34, 36, 37, 38, 39, 41,
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 56, 57, 58, 59, 59, 60, 60, 61, 61,
    62, 62, 62, 63, 63, 63, 64, 64, 64, 64, 64,
];

#[derive(Clone, Copy)]
struct Vertex3 {
    x: i8,
    y: i8,
    z: i8,
}

const POLY0_VTX: [Vertex3; 6] = [
    Vertex3 { x: 0, y: 65, z: 0 },
    Vertex3 { x: 0, y: -65, z: 0 },
    Vertex3 { x: 0, y: 0, z: -40 },
    Vertex3 { x: -40, y: 0, z: 0 },
    Vertex3 { x: 0, y: 0, z: 40 },
    Vertex3 { x: 40, y: 0, z: 0 },
];
const POLY0_POLYS: [u8; 40] = [
    3, 0, 5, 2, 4, 3, 0, 2, 3, 1, 3, 0, 3, 4, 2, 3, 0, 4, 5, 3, 3, 1, 2, 5, 4, 3, 1, 3, 2, 1, 3, 1,
    4, 3, 2, 3, 1, 5, 4, 3,
];
const POLY1_VTX: [Vertex3; 6] = [
    Vertex3 { x: 0, y: 40, z: 10 },
    Vertex3 {
        x: 40,
        y: -40,
        z: 10,
    },
    Vertex3 {
        x: -40,
        y: -40,
        z: 10,
    },
    Vertex3 {
        x: 0,
        y: 40,
        z: -10,
    },
    Vertex3 {
        x: -40,
        y: -40,
        z: -10,
    },
    Vertex3 {
        x: 40,
        y: -40,
        z: -10,
    },
];
const POLY1_POLYS: [u8; 28] = [
    3, 0, 1, 2, 7, 3, 3, 4, 5, 6, 4, 0, 3, 5, 1, 5, 4, 1, 5, 4, 2, 4, 4, 3, 0, 2, 4, 3,
];

struct PolyConfig {
    num_vtx: u8,
    num_poly: u8,
    vtx_val: u16,
    polys_val: u16,
    vertex: &'static [Vertex3],
    poly: &'static [u8],
}

const POLY_CONFIGS: [PolyConfig; 2] = [
    PolyConfig {
        num_vtx: 6,
        num_poly: 8,
        vtx_val: 0xff98,
        polys_val: 0xffaa,
        vertex: &POLY0_VTX,
        poly: &POLY0_POLYS,
    },
    PolyConfig {
        num_vtx: 6,
        num_poly: 5,
        vtx_val: 0xffd2,
        polys_val: 0xffe4,
        vertex: &POLY1_VTX,
        poly: &POLY1_POLYS,
    },
];
const RASTER_COLORS: [u32; 16] = [
    0x00, 0xff, 0xff00, 0xffff, 0xff0000, 0xff00ff, 0xffff00, 0xffffff, 0xff000000, 0xff0000ff,
    0xff00ff00, 0xff00ffff, 0xffff0000, 0xffff00ff, 0xffffff00, 0xffffffff,
];
const LEFT_SIDE_MASK: [u16; 8] = [
    0xffff, 0x7f7f, 0x3f3f, 0x1f1f, 0x0f0f, 0x0707, 0x0303, 0x0101,
];
const RIGHT_SIDE_MASK: [u16; 8] = [
    0x8080, 0xc0c0, 0xe0e0, 0xf0f0, 0xf8f8, 0xfcfc, 0xfefe, 0xffff,
];

/// Runs one complete clear, transform, projection, and rasterization pass.
pub fn run_frame(ram: &mut Ram<'_>) {
    empty_bitmap_buffer(ram);
    set_shape_pointer(ram);
    set_rotation_matrix(ram);
    operate_rotation(ram);
    draw_polyhedron(ram);
}

/// The original signed numerator / unsigned denominator fixed-point helper.
#[must_use]
pub fn divide(ram: &mut Ram<'_>, a: u16, b: u16) -> u16 {
    let negative = (a as i16) < 0;
    ram.write_u16(POLY_TMP1, if negative { a.wrapping_neg() } else { a });
    ram.write_u16(POLY_TMP0, b);
    while ram.read_u16(POLY_TMP0) >= 256 {
        ram.write_u16(POLY_TMP0, ram.read_u16(POLY_TMP0) >> 1);
        ram.write_u16(POLY_TMP1, ram.read_u16(POLY_TMP1) >> 1);
    }
    let q = ram.read_u16(POLY_TMP1) / ram.read_u16(POLY_TMP0);
    if negative { q.wrapping_neg() } else { q }
}

pub fn set_shape_pointer(ram: &mut Ram<'_>) {
    ram.write_u16(
        POLY_VAR1,
        u16::from(ram.read_u8(POLY_CONFIG1))
            .wrapping_mul(2)
            .wrapping_add(0x80),
    );
    ram.write_u16(
        POLY_TMP0,
        u16::from(ram.read_u8(POLY_WHICH_MODEL)).wrapping_mul(2),
    );
    let config = &POLY_CONFIGS[usize::from(ram.read_u8(POLY_WHICH_MODEL))];
    ram.write_u8(POLY_CONFIG_NUM_VERTEX, config.num_vtx);
    ram.write_u8(POLY_CONFIG_NUM_POLYS, config.num_poly);
    ram.write_u16(POLY_FROMLUT_PTR2, config.vtx_val);
    ram.write_u16(POLY_FROMLUT_PTR4, config.polys_val);
}

pub fn set_rotation_matrix(ram: &mut Ram<'_>) {
    let a = usize::from(ram.read_u8(POLY_A));
    let b = usize::from(ram.read_u8(POLY_B));
    ram.write_i16(POLY_SIN_A, i16::from(POLY_SIN_COS[a]));
    ram.write_i16(POLY_COS_A, i16::from(POLY_SIN_COS[a + 64]));
    ram.write_i16(POLY_SIN_B, i16::from(POLY_SIN_COS[b]));
    ram.write_i16(POLY_COS_B, i16::from(POLY_SIN_COS[b + 64]));
    let sin_a = i32::from(ram.read_i16(POLY_SIN_A) as i8);
    let cos_a = i32::from(ram.read_i16(POLY_COS_A) as i8);
    let sin_b = i32::from(ram.read_i16(POLY_SIN_B));
    let cos_b = i32::from(ram.read_i16(POLY_COS_B));
    ram.write_u16(POLY_E0, (((sin_b * sin_a) >> 8) * 4) as u16);
    ram.write_u16(POLY_E1, (((cos_b * cos_a) >> 8) * 4) as u16);
    ram.write_u16(POLY_E2, (((cos_b * sin_a) >> 8) * 4) as u16);
    ram.write_u16(POLY_E3, (((sin_b * cos_a) >> 8) * 4) as u16);
}

pub fn operate_rotation(ram: &mut Ram<'_>) {
    let config = &POLY_CONFIGS[usize::from(ram.read_u8(POLY_WHICH_MODEL))];
    let mut i = usize::from(ram.read_u8(POLY_CONFIG_NUM_VERTEX));
    do_loop(&mut i, |i| {
        let vertex = config.vertex[i];
        // The source walks the packed struct backwards and names z/y/x in reverse.
        ram.write_i8(POLY_FROMLUT_X, vertex.z);
        ram.write_i8(POLY_FROMLUT_Y, vertex.y);
        ram.write_i8(POLY_FROMLUT_Z, vertex.x);
        rotate_point(ram);
        project_point(ram);
        ram.write_u8(
            POLY_ARR_X + i,
            ram.read_u8(POLY_BASE_X)
                .wrapping_add(ram.read_u16(POLY_F0) as u8),
        );
        ram.write_u8(
            POLY_ARR_Y + i,
            ram.read_u8(POLY_BASE_Y)
                .wrapping_sub(ram.read_u16(POLY_F1) as u8),
        );
    });
}

fn do_loop(i: &mut usize, mut body: impl FnMut(usize)) {
    loop {
        *i -= 1;
        body(*i);
        if *i == 0 {
            break;
        }
    }
}

pub fn rotate_point(ram: &mut Ram<'_>) {
    let x = i32::from(ram.read_i8(POLY_FROMLUT_X));
    let y = i32::from(ram.read_i8(POLY_FROMLUT_Y));
    let z = i32::from(ram.read_i8(POLY_FROMLUT_Z));
    let sin_a = i32::from(ram.read_i16(POLY_SIN_A));
    let cos_a = i32::from(ram.read_i16(POLY_COS_A));
    let sin_b = i32::from(ram.read_i16(POLY_SIN_B));
    let cos_b = i32::from(ram.read_i16(POLY_COS_B));
    let e0 = i32::from(ram.read_i16(POLY_E0));
    let e1 = i32::from(ram.read_i16(POLY_E1));
    let e2 = i32::from(ram.read_i16(POLY_E2));
    let e3 = i32::from(ram.read_i16(POLY_E3));
    ram.write_u16(POLY_F0, (cos_b * z - sin_b * x) as u16);
    ram.write_u16(POLY_F1, (e0 * z + cos_a * y + e2 * x) as u16);
    ram.write_u16(
        POLY_F2,
        (((e3 * z) >> 8) - ((sin_a * y) >> 8)
            + ((e1 * x) >> 8)
            + i32::from(ram.read_u16(POLY_VAR1))) as u16,
    );
}

pub fn project_point(ram: &mut Ram<'_>) {
    let f0 = divide(ram, ram.read_u16(POLY_F0), ram.read_u16(POLY_F2));
    ram.write_u16(POLY_F0, f0);
    let f1 = divide(ram, ram.read_u16(POLY_F1), ram.read_u16(POLY_F2));
    ram.write_u16(POLY_F1, f1);
}

pub fn draw_polyhedron(ram: &mut Ram<'_>) {
    let config = &POLY_CONFIGS[usize::from(ram.read_u8(POLY_WHICH_MODEL))];
    let mut src = 0;
    loop {
        let count = config.poly[src];
        src += 1;
        ram.write_u8(POLY_NUM_VERTEX_IN_POLY, count);
        ram.write_u8(POLY_TMP0, count);
        ram.write_u8(POLY_XY_COORDS, count.wrapping_mul(2));
        let mut i = 1usize;
        loop {
            let j = usize::from(config.poly[src]);
            src += 1;
            ram.write_u8(POLY_XY_COORDS + i, ram.read_u8(POLY_ARR_X + j));
            ram.write_u8(POLY_XY_COORDS + i + 1, ram.read_u8(POLY_ARR_Y + j));
            i += 2;
            let left = ram.read_u8(POLY_TMP0).wrapping_sub(1);
            ram.write_u8(POLY_TMP0, left);
            if left == 0 {
                break;
            }
        }
        ram.write_u8(POLY_RASTER_COLOR_CONFIG, config.poly[src]);
        src += 1;
        if calculate_cross_product(ram) > 0 {
            set_foreground_color(ram);
            draw_face(ram);
        }
        let left = ram.read_u8(POLY_CONFIG_NUM_POLYS).wrapping_sub(1);
        ram.write_u8(POLY_CONFIG_NUM_POLYS, left);
        if left == 0 {
            break;
        }
    }
}

pub fn set_foreground_color(ram: &mut Ram<'_>) {
    let t = if ram.read_u8(POLY_WHICH_MODEL) != 0 {
        ram.read_u8(POLY_CONFIG1) >> 5
    } else {
        0
    };
    let a = ((u32::from(ram.read_u16(POLY_TMP0)) << (u32::from(t) + 1)) >> 8) as u8;
    set_color_mask(ram, a.clamp(1, 7));
}

#[must_use]
pub fn calculate_cross_product(ram: &mut Ram<'_>) -> i16 {
    let a = i16::from(ram.read_u8(POLY_XY_COORDS + 3)) - i16::from(ram.read_u8(POLY_XY_COORDS + 1));
    let rhs = ram
        .read_u8(POLY_XY_COORDS + 6)
        .wrapping_sub(ram.read_u8(POLY_XY_COORDS + 4)) as i8;
    let mut value = (i32::from(a) * i32::from(rhs)) as u16;
    let a = i16::from(ram.read_u8(POLY_XY_COORDS + 5)) - i16::from(ram.read_u8(POLY_XY_COORDS + 3));
    let rhs = ram
        .read_u8(POLY_XY_COORDS + 4)
        .wrapping_sub(ram.read_u8(POLY_XY_COORDS + 2)) as i8;
    value = value.wrapping_sub((i32::from(a) * i32::from(rhs)) as u16);
    ram.write_u16(POLY_TMP0, value);
    value as i16
}

pub fn set_color_mask(ram: &mut Ram<'_>, color: u8) {
    let value = RASTER_COLORS[usize::from(color)];
    ram.write_u16(POLY_RASTER_COLOR0, value as u16);
    ram.write_u16(POLY_RASTER_COLOR1, (value >> 16) as u16);
}

pub fn empty_bitmap_buffer(ram: &mut Ram<'_>) {
    ram.slice_mut(POLYHEDRAL_BUFFER, 0x800).fill(0);
}

pub fn draw_face(ram: &mut Ram<'_>) {
    let mut n = usize::from(ram.read_u8(POLY_XY_COORDS));
    let mut min_y = ram.read_u8(POLY_XY_COORDS + n);
    let mut min_idx = n;
    loop {
        n = n.wrapping_sub(2);
        if n == 0 {
            break;
        }
        let y = ram.read_u8(POLY_XY_COORDS + n);
        if y < min_y {
            min_y = y;
            min_idx = n;
        }
    }
    let row = ((u16::from(min_y & 0x38)) ^ if min_y & 0x20 != 0 { 0x24 } else { 0 }) << 6;
    ram.write_u16(
        POLY_RASTER_DST_PTR,
        0xe800u16
            .wrapping_add(row)
            .wrapping_add(u16::from(min_y & 7) * 2),
    );
    ram.write_u8(POLY_CUR_VERTEX_IDX0, min_idx as u8);
    ram.write_u8(POLY_CUR_VERTEX_IDX1, min_idx as u8);
    ram.write_u8(POLY_TOTAL_NUM_STEPS, ram.read_u8(POLY_XY_COORDS) >> 1);
    let y = ram.read_u8(POLY_XY_COORDS + min_idx);
    let x = ram.read_u8(POLY_XY_COORDS + min_idx - 1);
    ram.write_u8(POLY_Y0_CUR, y);
    ram.write_u8(POLY_Y1_CUR, y);
    ram.write_u8(POLY_X0_CUR, x);
    ram.write_u8(POLY_X1_CUR, x);
    if set_left(ram) || set_right(ram) {
        return;
    }
    loop {
        fill_line(ram);
        let ptr = ram.read_u16(POLY_RASTER_DST_PTR);
        if ptr as u8 != 0x0e {
            ram.write_u16(POLY_RASTER_DST_PTR, ptr.wrapping_add(2));
        } else {
            let a = ((ptr >> 8) as u8).wrapping_add(2);
            ram.write_u16(
                POLY_RASTER_DST_PTR,
                u16::from(a ^ if a & 8 != 0 { 0 } else { 0x19 }) << 8,
            );
        }
        if ram.read_u8(POLY_Y0_CUR) == ram.read_u8(POLY_Y0_TRIG) {
            ram.write_u8(POLY_X0_CUR, ram.read_u8(POLY_X0_TARGET));
            if set_left(ram) {
                return;
            }
        }
        ram.write_u8(POLY_Y0_CUR, ram.read_u8(POLY_Y0_CUR).wrapping_add(1));
        if ram.read_u8(POLY_Y1_CUR) == ram.read_u8(POLY_Y1_TRIG) {
            ram.write_u8(POLY_X1_CUR, ram.read_u8(POLY_X1_TARGET));
            if set_right(ram) {
                return;
            }
        }
        ram.write_u8(POLY_Y1_CUR, ram.read_u8(POLY_Y1_CUR).wrapping_add(1));
        ram.write_u16(
            POLY_X0_FRAC,
            ram.read_u16(POLY_X0_FRAC)
                .wrapping_add(ram.read_u16(POLY_X0_STEP)),
        );
        ram.write_u16(
            POLY_X1_FRAC,
            ram.read_u16(POLY_X1_FRAC)
                .wrapping_add(ram.read_u16(POLY_X1_STEP)),
        );
    }
}

pub fn fill_line(ram: &mut Ram<'_>) {
    let left = LEFT_SIDE_MASK[usize::from((ram.read_u16(POLY_X0_FRAC) >> 8) & 7)];
    let right = RIGHT_SIDE_MASK[usize::from((ram.read_u16(POLY_X1_FRAC) >> 8) & 7)];
    let tmp2 = ((ram.read_u16(POLY_X0_FRAC) >> 8) & 0x38) as u8;
    ram.write_u8(POLY_TMP2, tmp2);
    let right_offset = i16::from(((ram.read_u16(POLY_X1_FRAC) >> 8) & 0x38) as u8);
    let mut ptr =
        usize::from(ram.read_u16(POLY_RASTER_DST_PTR)) + usize::from(right_offset as u16) * 4;
    let delta = right_offset - i16::from(tmp2);
    if delta == 0 {
        let mask = left & right;
        ram.write_u16(POLY_TMP1, mask);
        masked_write(ram, ptr, POLY_RASTER_COLOR0, mask);
        masked_write(ram, ptr + 16, POLY_RASTER_COLOR1, mask);
        return;
    }
    if delta < 0 {
        return;
    }
    let mut n = delta >> 3;
    masked_write(ram, ptr, POLY_RASTER_COLOR0, right);
    masked_write(ram, ptr + 16, POLY_RASTER_COLOR1, right);
    ptr -= 32;
    loop {
        n -= 1;
        if n == 0 {
            break;
        }
        ram.write_u16(ptr, ram.read_u16(POLY_RASTER_COLOR0));
        ram.write_u16(ptr + 16, ram.read_u16(POLY_RASTER_COLOR1));
        ptr -= 32;
    }
    masked_write(ram, ptr, POLY_RASTER_COLOR0, left);
    masked_write(ram, ptr + 16, POLY_RASTER_COLOR1, left);
    ram.write_u16(POLY_TMP1, left);
    ram.write_u16(POLY_RASTER_NUMFULL, 0);
}

fn masked_write(ram: &mut Ram<'_>, address: usize, color_address: usize, mask: u16) {
    let old = ram.read_u16(address);
    let color = ram.read_u16(color_address);
    ram.write_u16(address, old ^ ((old ^ color) & mask));
}

pub fn set_left(ram: &mut Ram<'_>) -> bool {
    set_edge(ram, true)
}
pub fn set_right(ram: &mut Ram<'_>) -> bool {
    set_edge(ram, false)
}

fn set_edge(ram: &mut Ram<'_>, left: bool) -> bool {
    let (idx_addr, x_cur_addr, y_cur_addr, x_target_addr, y_trig_addr, frac_addr, step_addr) =
        if left {
            (
                POLY_CUR_VERTEX_IDX0,
                POLY_X0_CUR,
                POLY_Y0_CUR,
                POLY_X0_TARGET,
                POLY_Y0_TRIG,
                POLY_X0_FRAC,
                POLY_X0_STEP,
            )
        } else {
            (
                POLY_CUR_VERTEX_IDX1,
                POLY_X1_CUR,
                POLY_Y1_CUR,
                POLY_X1_TARGET,
                POLY_Y1_TRIG,
                POLY_X1_FRAC,
                POLY_X1_STEP,
            )
        };
    let i = loop {
        let steps = ram.read_u8(POLY_TOTAL_NUM_STEPS).wrapping_sub(1);
        ram.write_u8(POLY_TOTAL_NUM_STEPS, steps);
        if (steps as i8) < 0 {
            return true;
        }
        let mut i = usize::from(ram.read_u8(idx_addr));
        if left {
            i = i.wrapping_sub(2);
            if i == 0 {
                i = usize::from(ram.read_u8(POLY_XY_COORDS));
            }
        } else {
            if i == usize::from(ram.read_u8(POLY_XY_COORDS)) {
                i = 0;
            }
            i += 2;
        }
        let y = ram.read_u8(POLY_XY_COORDS + i);
        if y < ram.read_u8(y_cur_addr) {
            return true;
        }
        if y != ram.read_u8(y_cur_addr) {
            break i;
        }
        ram.write_u8(x_cur_addr, ram.read_u8(POLY_XY_COORDS + i - 1));
        ram.write_u8(idx_addr, i as u8);
    };
    ram.write_u8(y_trig_addr, ram.read_u8(POLY_XY_COORDS + i));
    ram.write_u8(x_target_addr, ram.read_u8(POLY_XY_COORDS + i - 1));
    ram.write_u8(idx_addr, i as u8);
    let signed_delta = i16::from(ram.read_u8(x_target_addr)) - i16::from(ram.read_u8(x_cur_addr));
    let magnitude = signed_delta.unsigned_abs();
    let dy = ram
        .read_u8(y_trig_addr)
        .wrapping_sub(ram.read_u8(y_cur_addr));
    let step = ((magnitude & 0xff) << 8) / u16::from(dy);
    ram.write_u16(frac_addr, (u16::from(ram.read_u8(x_cur_addr)) << 8) | 0x80);
    ram.write_u16(
        step_addr,
        if signed_delta < 0 {
            step.wrapping_neg()
        } else {
            step
        },
    );
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn divide_preserves_signed_result_and_scales_large_denominators() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        assert_eq!(divide(&mut ram, 1000, 10), 100);
        assert_eq!(divide(&mut ram, (-1000i16) as u16, 10), (-100i16) as u16);
        assert_eq!(divide(&mut ram, 0x7000, 0x0200), 56);
        assert_eq!(ram.read_u16(POLY_TMP0), 0x0080);
        assert_eq!(ram.read_u16(POLY_TMP1), 0x1c00);
    }

    #[test]
    fn fill_line_writes_masked_edges_and_full_middle_tile() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(POLY_RASTER_DST_PTR, 0xe800);
        ram.write_u16(POLY_X0_FRAC, 1 << 8);
        ram.write_u16(POLY_X1_FRAC, 18 << 8);
        ram.write_u16(POLY_RASTER_COLOR0, 0xffff);
        ram.write_u16(POLY_RASTER_COLOR1, 0x00ff);
        fill_line(&mut ram);
        assert_eq!(ram.read_u16(0xe800), 0x7f7f);
        assert_eq!(ram.read_u16(0xe820), 0xffff);
        assert_eq!(ram.read_u16(0xe840), 0xe0e0);
        assert_eq!(ram.read_u16(0xe810), 0x007f);
        assert_eq!(ram.read_u16(0xe830), 0x00ff);
        assert_eq!(ram.read_u16(0xe850), 0x00e0);
    }

    #[test]
    fn frame_rasterizes_model_zero_and_clears_stale_bitmap_data() {
        let mut bytes = [0xaa; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u8(POLY_CONFIG1, 0);
        ram.write_u8(POLY_WHICH_MODEL, 0);
        ram.write_u8(POLY_A, 0);
        ram.write_u8(POLY_B, 0);
        ram.write_u8(POLY_BASE_X, 64);
        ram.write_u8(POLY_BASE_Y, 64);
        run_frame(&mut ram);
        let signature = ram
            .slice(POLYHEDRAL_BUFFER, 0x800)
            .iter()
            .fold((0usize, 0u32), |(count, sum), &byte| {
                (count + usize::from(byte != 0), sum + u32::from(byte))
            });
        assert_eq!(signature, (240, 56_648));
        assert!(
            ram.slice(POLYHEDRAL_BUFFER, 0x800)
                .iter()
                .any(|&byte| byte != 0)
        );
        assert!(ram.slice(POLYHEDRAL_BUFFER, 0x800).contains(&0));
        assert_eq!(ram.read_u8(POLY_CONFIG_NUM_POLYS), 0);
        assert_eq!(ram.slice(POLY_ARR_X, 6), &[64, 64, 64, 44, 64, 84]);
    }
}
