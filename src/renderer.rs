use cgmath::{Matrix4, Point3, Vector4, prelude::*};
use cgmath::{PerspectiveFov, Vector2, Vector3};

use crate::window::RawWindowBitmap;

type Vec3f = Vector3<f32>;
type Vec2f = Vector2<f32>;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

struct Camera {
    projection: PerspectiveFov<f32>,
    position: Vec3f,
    forward: Vec3f,
    up: Vec3f,
    view_matrix: Matrix4<f32>,
}

struct Sphere {
    position: Vec3f,
    size: f32,
}

pub struct Renderer {
    camera: Camera,
    spheres: Vec<Sphere>,
    frame: usize,
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self {
            a: (value & 0xFF000000 >> 24) as u8,
            r: (value >> 8 & 0x00FF0000 >> 16) as u8,
            g: (value >> 16 & 0x0000FF00 >> 8) as u8,
            b: (value >> 24 & 0x000000FF) as u8,
        }
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        let mut acc: u32 = 0;
        acc += (value.a as u32) << 24;
        acc += (value.r as u32) << 16;
        acc += (value.g as u32) << 8;
        acc += value.b as u32;
        acc
    }
}

impl Camera {
    pub fn new(position: Vec3f, forward: Vec3f, up: Vec3f) -> Self {
        let projection = PerspectiveFov {
            fovy: cgmath::Rad::from(cgmath::Deg(70f32)),
            aspect: 1.0,
            near: 0.1f32,
            far: 100f32,
        };
        let view_matrix = Matrix4::look_to_rh(Point3::from_vec(position), forward, up);

        Self {
            projection,
            position,
            forward,
            up,
            view_matrix,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.projection.aspect = width as f32 / height as f32;
    }

    pub fn set_position(&mut self, position: Vec3f, forward: Vec3f, up: Vec3f) {
        let view_matrix = Matrix4::look_to_rh(Point3::from_vec(position), forward, up);
        self.position = position;
        self.view_matrix = view_matrix;
    }
}

impl Sphere {
    pub fn new(position: Vec3f, size: f32) -> Self {
        Self { position, size }
    }

    pub fn intersect(&self, origin: Vec3f, ray: Vec3f) -> Option<Vec3f> {
        let origin_center = origin - self.position;
        let a = ray.dot(ray);
        let b = origin_center.dot(ray);
        let c = origin_center.dot(origin_center) - self.size.powi(2);
        let discriminant = b * b - a * c;

        if discriminant > 0.0 {
            let tmp = (-b - discriminant.sqrt()) / a;

            if tmp < 10000f32 && tmp > 0.00001 {
                return Some(origin + tmp * ray);
            }
        }

        None
    }
}

fn vec2_abs(input: &Vec2f) -> Vec2f {
    Vec2f {
        x: input.x.abs(),
        y: input.y.abs(),
    }
}

fn vec3_abs(input: &Vec3f) -> Vec3f {
    Vec3f {
        x: input.x.abs(),
        y: input.y.abs(),
        z: input.z.abs(),
    }
}

fn vec3_max(a: Vec3f, b: f32) -> Vec3f {
    Vec3f {
        x: a.x.max(b),
        y: a.y.max(b),
        z: a.z.max(b),
    }
}

fn sdf_sphere(position: Vec3f, size: f32) -> f32 {
    return position.magnitude() - size;
}

fn sdf_box(position: Vec3f, bounds: Vec3f) -> f32 {
    let abs_pos = vec3_abs(&position);
    let d = abs_pos - bounds;
    let d_max = vec3_max(d, 0.0).magnitude();
    d.x.max(d.y.max(d.z)).min(0.0) + d_max
}

#[inline]
fn op_u(a: Vec2f, b: Vec2f) -> Vec2f {
    if a.x < b.x {
        return a;
    }

    b
}

fn sdf_scene(position: Vec3f) -> Vec2f {
    let mut res = Vec2f {
        x: position.y,
        y: 0.0,
    };

    let sphere_position = Vec3f {
        x: 0.0,
        y: -5.0,
        z: 8.0,
    };
    let sphere = sdf_sphere(position + sphere_position, 2.0);
    res = op_u(res, Vec2f { x: sphere, y: 15.0 });

    let box_position = Vec3f {
        x: 1.0,
        y: -4.0,
        z: 6.0,
    };
    let box_bounds = Vec3f {
        x: 0.2,
        y: 0.2,
        z: 0.8,
    };
    let box_sdf = sdf_box(position + box_position, box_bounds);
    res = op_u(res, Vec2f { x: box_sdf, y: 3.0 });

    res
}

fn sdf_raycast(origin: Vec3f, direction: Vec3f) -> Vec2f {
    let mut res = Vec2f { x: -1.0, y: -1.0 };

    let tmin = 1.0f32;
    let tmax = 10.0f32;

    let mut t = tmin;

    for _ in 0..50 {
        if t >= tmax {
            break;
        }

        let h = sdf_scene(origin + direction * t);

        if vec2_abs(&h).x < 0.0001 * t {
            res = Vec2f { x: t, y: h.y };
            break;
        }

        t += h.x;
    }

    res
}

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new(
            Vec3f::new(0.0, 3.0, 0.0),
            Vec3f::new(0.0, 0.0, -1.0),
            Vec3f::new(0.0, 1.0, 0.0),
        );
        let spheres = vec![Sphere::new(Vec3f::new(10.0, 0.0, -15.0), 2f32)];
        Self {
            camera,
            spheres,
            frame: 0,
        }
    }

    pub fn resize(&mut self, bitmap: &mut RawWindowBitmap) {
        self.camera.resize(bitmap.width, bitmap.height);
    }

    pub fn render_scene(&mut self, bitmap: &mut RawWindowBitmap) {
        self.frame += 1;

        self.camera.set_position(
            self.camera.position + Vec3f::unit_y() * (self.frame as f32 * 0.15).sin() * 0.2f32,
            self.camera.forward + Vec3f::unit_x() * (self.frame as f32 * 0.25).sin() * 0.5f32,
            self.camera.up,
        );

        let projmat: Matrix4<f32> = self.camera.projection.into();
        let inv_proj = projmat.invert().unwrap();
        let inv_view = self.camera.view_matrix.invert().unwrap();

        for y in 0..bitmap.height {
            for x in 0..bitmap.width {
                let ndc_x = (2.0 * x as f32) / bitmap.width as f32 - 1.0;
                let ndc_y = 1.0 - (2.0 * y as f32) / bitmap.height as f32;
                let ray = Vector4::<f32>::new(ndc_x, ndc_y, -1f32, 1.0);
                let mut eye_ray = inv_proj * ray;
                eye_ray.z = -1.0;
                eye_ray.w = 0.0;

                let ray_world = inv_view * eye_ray;
                let ray_dir = ray_world.truncate().normalize();

                let raycast = sdf_raycast(self.camera.position, ray_dir);
                let mut color: u32 = 0x00000000;

                let t = raycast.x;
                let m = raycast.y;

                if m > -0.5 {
                    color = Color {
                        a: 0,
                        r: 2 * ((m.abs() * 2.0) as u8),
                        g: 4 * ((m.abs() * 2.0) as u8),
                        b: 6 * ((m.abs() * 2.0) as u8),
                    }
                    .into();
                }

                bitmap.set_pixel(x as usize, y as usize, color.into());
            }
        }
    }
}
