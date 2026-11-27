use crate::worldgen::rng::RandomSource;

const PERM_SIZE: usize = 256;
const PERM_MASK: i32 = PERM_SIZE as i32 - 1;
const PERM_SHIFT: i32 = 8;

const ENABLE_FARLANDS: bool = false;

const ROUND_OFF: f64 = 3.3554432e7;
const HALF_ROUND_OFF: f64 = ROUND_OFF / 2.0;

type PermsType = [u8; PERM_SIZE];

pub struct PerlinNoise {
    perms: PermsType,
    offset_x: f64,
    offset_y: f64,
    offset_z: f64,
}

impl PerlinNoise {
    pub fn new(random: &mut RandomSource) -> Self {
        Self {
            offset_x: random.next_double() * 256.0,
            offset_y: random.next_double() * 256.0,
            offset_z: random.next_double() * 256.0,
            perms: generate_perms(random),
        }
    }

    fn p(&self, i: i32) -> i32 {
        self.perms[(i & PERM_MASK) as usize] as i32 & PERM_MASK
    }

    fn hash(&self, x: i32, y: i32, z: i32) -> i32 {
        self.p(self.p(self.p(x) + y) + z)
    }

    pub fn noise(&self, mut x: f64, mut y: f64, mut z: f64, derivative_out: &mut [f32; 3]) -> f32 {
        x = warp(x) + self.offset_x;
        y = warp(y) + self.offset_y;
        z = warp(z) + self.offset_z;

        let floor_x = floor(x) as i32;
        let floor_y = floor(y) as i32;
        let floor_z = floor(z) as i32;

        let releative_x = (x - floor_x as f64) as f32;
        let releative_y = (y - floor_y as f64) as f32;
        let releative_z = (z - floor_z as f64) as f32;

        let mut deriv = [0.0; 3];
        let value = self.sample_with_derivative(
            floor_x,
            floor_y,
            floor_z,
            releative_x,
            releative_y,
            releative_z,
            &mut deriv,
        );
        *derivative_out = deriv;
        value
    }

    fn sample_and_lerp(
        &self,
        x: i32,
        y: i32,
        z: i32,
        releative_x: f32,
        releative_y: f32,
        releative_z: f32,
        original_releative_y: f32,
    ) -> f32 {
        let x0 = self.permute(x);
        let x1 = self.permute(x + 1);

        let xy00 = self.permute(x0 + y);
        let xy01 = self.permute(x0 + y + 1);
        let xy10 = self.permute(x1 + y);
        let xy11 = self.permute(x1 + y + 1);

        let d000 = grad_dot(
            self.permute(xy00 + z),
            releative_x,
            releative_y,
            releative_z,
        );
        let d100 = grad_dot(
            self.permute(xy10 + z),
            releative_x - 1.0,
            releative_y,
            releative_z,
        );
        let d010 = grad_dot(
            self.permute(xy01 + z),
            releative_x,
            releative_y - 1.0,
            releative_z,
        );
        let d110 = grad_dot(
            self.permute(xy11 + z),
            releative_x - 1.0,
            releative_y - 1.0,
            releative_z,
        );
        let d001 = grad_dot(
            self.permute(xy00 + z + 1),
            releative_x,
            releative_y,
            releative_z - 1.0,
        );
        let d101 = grad_dot(
            self.permute(xy10 + z + 1),
            releative_x - 1.0,
            releative_y,
            releative_z - 1.0,
        );
        let d011 = grad_dot(
            self.permute(xy01 + z + 1),
            releative_x,
            releative_y - 1.0,
            releative_z - 1.0,
        );
        let d111 = grad_dot(
            self.permute(xy11 + z + 1),
            releative_x - 1.0,
            releative_y - 1.0,
            releative_z - 1.0,
        );

        let x_alpha = smoothstep(releative_x);
        let y_alpha = smoothstep(original_releative_y);
        let z_alpha = smoothstep(releative_z);

        lerp3_f32(
            x_alpha, y_alpha, z_alpha, d000, d100, d010, d110, d001, d101, d011, d111,
        )
    }

    fn sample_with_derivative(
        &self,
        x: i32,
        y: i32,
        z: i32,
        xr: f32,
        yr: f32,
        zr: f32,
        derivative_out: &mut [f32; 3],
    ) -> f32 {
        let x0 = self.permute(x);
        let x1 = self.permute(x + 1);

        let xy00 = self.permute(x0 + y);
        let xy01 = self.permute(x0 + y + 1);
        let xy10 = self.permute(x1 + y);
        let xy11 = self.permute(x1 + y + 1);

        let h000 = self.permute(xy00 + z);
        let h100 = self.permute(xy10 + z);
        let h010 = self.permute(xy01 + z);
        let h110 = self.permute(xy11 + z);
        let h001 = self.permute(xy00 + z + 1);
        let h101 = self.permute(xy10 + z + 1);
        let h011 = self.permute(xy01 + z + 1);
        let h111 = self.permute(xy11 + z + 1);

        let g000 = GRADIENT[(h000 & 15) as usize];
        let g100 = GRADIENT[(h100 & 15) as usize];
        let g010 = GRADIENT[(h010 & 15) as usize];
        let g110 = GRADIENT[(h110 & 15) as usize];
        let g001 = GRADIENT[(h001 & 15) as usize];
        let g101 = GRADIENT[(h101 & 15) as usize];
        let g011 = GRADIENT[(h011 & 15) as usize];
        let g111 = GRADIENT[(h111 & 15) as usize];

        let d000 = self.permute2grad(h000, xr, yr, zr);
        let d100 = self.permute2grad(h100, xr - 1.0, yr, zr);
        let d010 = self.permute2grad(h010, xr, yr - 1.0, zr);
        let d110 = self.permute2grad(h110, xr - 1.0, yr - 1.0, zr);
        let d001 = self.permute2grad(h001, xr, yr, zr - 1.0);
        let d101 = self.permute2grad(h101, xr - 1.0, yr, zr - 1.0);
        let d011 = self.permute2grad(h011, xr, yr - 1.0, zr - 1.0);
        let d111 = self.permute2grad(h111, xr - 1.0, yr - 1.0, zr - 1.0);

        let x_alpha = smoothstep(xr);
        let y_alpha = smoothstep(yr);
        let z_alpha = smoothstep(zr);

        let dx_alpha = smoothstep_deriv(xr);
        let dy_alpha = smoothstep_deriv(yr);
        let dz_alpha = smoothstep_deriv(zr);

        let value = lerp3_f32(
            x_alpha, y_alpha, z_alpha,
            d000, d100, d010, d110,
            d001, d101, d011, d111,
        );

        let by0 = lerp_f32(x_alpha, d000, d100);
        let by1 = lerp_f32(x_alpha, d010, d110);
        let bottom = lerp_f32(y_alpha, by0, by1);

        let ty0 = lerp_f32(x_alpha, d001, d101);
        let ty1 = lerp_f32(x_alpha, d011, d111);
        let top = lerp_f32(y_alpha, ty0, ty1);

        let d_by0_dx = dx_alpha * (d100 - d000) + x_alpha * (g100[0] - g000[0]);
        let d_by1_dx = dx_alpha * (d110 - d010) + x_alpha * (g110[0] - g010[0]);
        let d_bottom_dx = lerp_f32(y_alpha, d_by0_dx, d_by1_dx);

        let d_ty0_dx = dx_alpha * (d101 - d001) + x_alpha * (g101[0] - g001[0]);
        let d_ty1_dx = dx_alpha * (d111 - d011) + x_alpha * (g111[0] - g011[0]);
        let d_top_dx = lerp_f32(y_alpha, d_ty0_dx, d_ty1_dx);

        let d_dx = lerp_f32(z_alpha, d_bottom_dx, d_top_dx);

        let d_by0_dy = g000[1] + x_alpha * (g100[1] - g000[1]);
        let d_by1_dy = g010[1] + x_alpha * (g110[1] - g010[1]);
        let d_bottom_dy = dy_alpha * (by1 - by0) + y_alpha * (d_by1_dy - d_by0_dy);

        let d_ty0_dy = g001[1] + x_alpha * (g101[1] - g001[1]);
        let d_ty1_dy = g011[1] + x_alpha * (g111[1] - g011[1]);
        let d_top_dy = dy_alpha * (ty1 - ty0) + y_alpha * (d_ty1_dy - d_ty0_dy);

        let d_dy = lerp_f32(z_alpha, d_bottom_dy, d_top_dy);

        let d_by0_dz = g000[2] + x_alpha * (g100[2] - g000[2]);
        let d_by1_dz = g010[2] + x_alpha * (g110[2] - g010[2]);
        let d_bottom_dz = lerp_f32(y_alpha, d_by0_dz, d_by1_dz);

        let d_ty0_dz = g001[2] + x_alpha * (g101[2] - g001[2]);
        let d_ty1_dz = g011[2] + x_alpha * (g111[2] - g011[2]);
        let d_top_dz = lerp_f32(y_alpha, d_ty0_dz, d_ty1_dz);

        let d_dz = dz_alpha * (top - bottom) + z_alpha * (d_top_dz - d_bottom_dz);

        derivative_out[0] = d_dx;
        derivative_out[1] = d_dy;
        derivative_out[2] = d_dz;

        value
    }

    fn permute(&self, x: i32) -> i32 {
        self.perms[(x & 0xFF) as usize] as i32 & 0xFF
    }

    fn permute2grad(&self, index: i32, dx: f32, dy: f32, dz: f32) -> f32 {
        grad_dot(self.permute(index), dx, dy, dz)
    }
}

const GRADIENT: [[f32; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

fn grad_dot(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let [gx, gy, gz] = GRADIENT[(hash & 15) as usize];
    gx * x + gy * y + gz * z
}

fn generate_perms(random: &mut RandomSource) -> PermsType {
    let mut perms = [0; PERM_SIZE];

    for i in 0..PERM_SIZE as usize {
        perms[i] = i as u8;
    }

    for i in 0..PERM_SIZE as usize {
        let offset = random.next_int_bounded((PERM_SIZE - i) as i32) as usize;
        perms.swap(i, i + offset);
    }

    perms
}

fn warp(x: f64) -> f64 {
    if (ENABLE_FARLANDS) {
        return x;
    }

    if x >= -HALF_ROUND_OFF && x < HALF_ROUND_OFF {
        x
    } else {
        x - (x / ROUND_OFF + 0.5).floor() * ROUND_OFF
    }
}

fn smoothstep(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn smoothstep_deriv(t: f32) -> f32 {
    // d/dt of 6t^5 - 15t^4 + 10t^3 = 30t^4 - 60t^3 + 30t^2 = 30 * t^2 * (t - 1)^2
    let t2 = t * t;
    let t1m = t - 1.0;
    30.0 * t2 * t1m * t1m
}

fn floor(x: f64) -> i32 {
    let i = x as i32;

    if x < i as f64 { i.wrapping_sub(1) } else { i }
}

fn lerp_f32(alpha: f32, p0: f32, p1: f32) -> f32 {
    p0 + alpha * (p1 - p0)
}

fn lerp2_f32(alpha1: f32, alpha2: f32, x00: f32, x10: f32, x01: f32, x11: f32) -> f32 {
    lerp_f32(
        alpha2,
        lerp_f32(alpha1, x00, x10),
        lerp_f32(alpha1, x01, x11),
    )
}

fn lerp3_f32(
    alpha1: f32,
    alpha2: f32,
    alpha3: f32,
    x000: f32,
    x100: f32,
    x010: f32,
    x110: f32,
    x001: f32,
    x101: f32,
    x011: f32,
    x111: f32,
) -> f32 {
    lerp_f32(
        alpha3,
        lerp2_f32(alpha1, alpha2, x000, x100, x010, x110),
        lerp2_f32(alpha1, alpha2, x001, x101, x011, x111),
    )
}
