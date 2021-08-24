use std::sync::mpsc::{self, Receiver};

use glam;

use crate::input::{InputState, Key};

pub const UP: glam::Vec3 = glam::const_vec3!([0.0, 1.0, 0.0]);
pub struct Camera {
    pub(crate) pos: glam::Vec3,
    pub(crate) dir: glam::Vec3,
    yaw: f32,
    pitch: f32,
    receiver: Receiver<(i32, i32)>,
}

impl Camera {
    const SPEED: f32 = 2.568;
    const MOUSE_SENSITIVITY: f32 = 0.687;

    pub fn move_dir(&self, key: &Key) -> glam::Vec3 {
        match key {
            Key::W => glam::vec3(self.dir.x, 0.0, self.dir.z).normalize(),
            Key::A => UP.cross(self.move_dir(&Key::W)),
            Key::S => -1.0 * self.move_dir(&Key::W),
            Key::D => -1.0 * self.move_dir(&Key::A),
            Key::Space => UP,
            Key::LShift => -UP,
        }
    }

    pub fn new(input: &InputState) -> Self {
        let (sender, receiver) = mpsc::channel();

        input.add_mouse_cb(move |dx, dy| {
            sender.send((dx, dy)).unwrap();
        });

        Camera {
            pos: glam::vec3(0.0, 0.0, 1.0),
            dir: glam::vec3(0.0, 0.0, -1.0),
            yaw: 0.0,
            pitch: 0.0,
            receiver,
        }
    }

    pub fn to_matrix(&self) -> glam::Mat4 {
        // TODO: Caching system
        let projection = glam::Mat4::perspective_rh_gl(45.0f32.to_radians(), 6.0 / 4.0, 0.1, 100.0);
        let view = glam::Mat4::look_at_rh(self.pos, self.pos + self.dir, UP);
        projection * view
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        for k in Key::KEYS.iter() {
            if input.is_pressed(k) {
                self.pos += dt * Self::SPEED * self.move_dir(k);
            }
        }

        let mut recompute = false;
        if let Some((dx, dy)) = self
            .receiver
            .try_iter()
            .reduce(|(ax, ay), (bx, by)| (ax + bx, ay + by))
        {
            // Compute new dir
            self.yaw += Self::MOUSE_SENSITIVITY * dx as f32;
            self.pitch += Self::MOUSE_SENSITIVITY * dy as f32;
            self.pitch = self.pitch.clamp(-89.9, 89.9);

            recompute = true;
        }

        if recompute {
            let pitch = self.pitch.to_radians();
            let yaw = self.yaw.to_radians();
            let xz_l = pitch.cos();
            self.dir = glam::vec3(xz_l * yaw.cos(), pitch.sin(), xz_l * yaw.sin());
        }
    }
}
