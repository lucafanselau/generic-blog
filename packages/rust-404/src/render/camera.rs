use std::sync::{Arc, RwLock};

use glam;

use crate::input::{InputState, Key};

pub struct Camera {
    pub pos: glam::Vec3,
    dir: glam::Vec3,
    yaw: Arc<RwLock<f32>>,
    pitch: Arc<RwLock<f32>>,
}

impl Camera {
    const UP: glam::Vec3 = glam::const_vec3!([0.0, 1.0, 0.0]);
    const SPEED: f32 = 1.568;
    const MOUSE_SENSITIVITY: f32 = 0.687;

    pub fn move_dir(&self, key: &Key) -> glam::Vec3 {
        match key {
            Key::W => self.dir,
            Key::A => Self::UP.cross(self.dir),
            Key::S => -1.0 * self.dir,
            Key::D => -1.0 * Self::UP.cross(self.dir),
        }
    }

    pub fn new(input: &InputState) -> Self {
        let yaw: Arc<RwLock<f32>> = Default::default();
        let pitch: Arc<RwLock<f32>> = Default::default();
        {
            let yaw = yaw.clone();
            let pitch = pitch.clone();
            input.add_mouse_cb(move |dx, dy| {
                *yaw.write().unwrap() += Self::MOUSE_SENSITIVITY * dx as f32;
                *pitch.write().unwrap() += Self::MOUSE_SENSITIVITY * dy as f32;
            });
        }

        Camera {
            pos: glam::vec3(0.0, 0.0, 1.0),
            dir: glam::vec3(0.0, 0.0, -1.0),
            yaw,
            pitch,
        }
    }

    pub fn to_matrix(&self) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh_gl(45.0f32.to_radians(), 6.0 / 4.0, 0.1, 10.0);
        let view = glam::Mat4::look_at_rh(self.pos, self.pos + self.dir, Self::UP);
        projection * view
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        for k in Key::KEYS.iter() {
            if input.is_pressed(k) {
                self.pos += dt * Self::SPEED * self.move_dir(k);
            }
        }

        // Compute new dir
        let pitch = self.pitch.read().unwrap().to_radians();
        let yaw = self.yaw.read().unwrap().to_radians();
        let xz_l = pitch.cos();
        self.dir = glam::vec3(xz_l * yaw.cos(), pitch.sin(), xz_l * yaw.sin());
    }
}
