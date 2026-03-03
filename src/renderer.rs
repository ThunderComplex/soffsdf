use cgmath::{Matrix4, Point3, Vector4, prelude::*};
use cgmath::{PerspectiveFov, Vector3};

use crate::window::RawWindowBitmap;

type Vec3f = Vector3<f32>;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
            let mut tmp = (-b - discriminant.sqrt()) / a;

            if tmp < 10000f32 && tmp > 0.00001 {
                return Some(origin + tmp * ray);
            }

            // tmp = (-b + discriminant.sqrt()) / a;

            // if tmp < 10000f32 && tmp > 0.00001 {
            //     return Some(origin + tmp * ray);
            // }
        }

        None
    }
}

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new(
            Vec3f::new(0.0, 0.0, 0.0),
            Vec3f::new(0.0, 0.0, -1.0),
            Vec3f::new(0.0, 1.0, 0.0),
        );
        let spheres = vec![Sphere::new(Vec3f::new(10.0, 0.0, -15.0), 2f32)];
        Self { camera, spheres }
    }

    pub fn resize(&mut self, bitmap: &mut RawWindowBitmap) {
        self.camera.resize(bitmap.width, bitmap.height);
    }

    pub fn render_scene(&self, bitmap: &mut RawWindowBitmap) {
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

                for sphere in self.spheres.iter() {
                    if let Some(intersection) = sphere.intersect(self.camera.position, ray_dir) {
                        let hit_normal = sphere.position - intersection;
                        let dot = hit_normal.normalize().dot(ray_dir);
                        let color = Color {
                            a: 255,
                            r: (((dot + 1.0) * 0.5) * 255.0) as u8,
                            g: (((dot + 1.0) * 0.5) * 255.0) as u8,
                            b: (((dot + 1.0) * 0.5) * 255.0) as u8,
                        };
                        bitmap.set_pixel(x as usize, y as usize, color.into());
                    }
                }
            }
        }
    }
}
