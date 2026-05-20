use crate::engine::terrain;
use crate::state::WorldParams;

#[derive(Clone, Copy, PartialEq)]
pub enum TourMode {
    None,
    Orbit,
    Scenic,
}

pub struct TourUpdate {
    pub pos: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
}

pub struct TourState {
    pub active: TourMode,
    pub timer: f64,
    pub orbit_angle: f64,
    pub orbit_pitch: f64,
    pub orbit_radius: f64,
    pub speed: f64,
    pub yaw_base: f64,
    pub target_yaw: f64,
    pub yaw_timer: f64,
    pub pause_timer: f64,
    pub look_at_structures: bool,
}

impl TourState {
    pub fn new() -> Self {
        Self {
            active: TourMode::None,
            timer: 0.0,
            orbit_angle: 0.0,
            orbit_pitch: -0.2,
            orbit_radius: 20.0,
            speed: 8.0,
            yaw_base: 0.0,
            target_yaw: 0.0,
            yaw_timer: 0.0,
            pause_timer: 0.0,
            look_at_structures: true,
        }
    }

    pub fn start_orbit(&mut self, yaw: f64, pitch: f64, radius: f64) {
        self.active = TourMode::Orbit;
        self.timer = 0.0;
        self.orbit_angle = yaw;
        self.orbit_pitch = pitch.max(-0.8).min(-0.1);
        self.orbit_radius = radius;
    }

    pub fn start_scenic(&mut self, yaw: f64, speed: f64) {
        self.active = TourMode::Scenic;
        self.timer = 0.0;
        self.speed = speed;
        self.yaw_base = yaw;
        self.target_yaw = yaw;
        self.yaw_timer = 0.0;
        self.pause_timer = 0.0;
    }

    pub fn stop(&mut self) {
        self.active = TourMode::None;
        self.timer = 0.0;
    }

    pub fn update(
        &mut self,
        delta: f64,
        params: &WorldParams,
        pos: &[f64; 3],
        yaw: f64,
        pitch: f64,
    ) -> Option<TourUpdate> {
        if self.active == TourMode::None {
            return None;
        }

        self.timer += delta;

        match self.active {
            TourMode::Orbit => {
                let speed_rad = 0.15 * delta;
                self.orbit_angle += speed_rad;

                let py = self.orbit_pitch;
                let r = self.orbit_radius;
                let ox = pos[0];
                let oy = pos[1];
                let oz = pos[2];

                let cam_x = ox + r * py.cos() * self.orbit_angle.sin();
                let cam_y = oy + r * py.sin();
                let cam_z = oz + r * py.cos() * self.orbit_angle.cos();

                Some(TourUpdate {
                    pos: [cam_x, cam_y, cam_z],
                    yaw: self.orbit_angle,
                    pitch: py,
                })
            }
            TourMode::Scenic => {
                self.yaw_timer += delta;

                if self.pause_timer > 0.0 {
                    self.pause_timer -= delta;
                    return None;
                }

                if self.yaw_timer > 4.0 {
                    self.yaw_timer = 0.0;
                    self.target_yaw = self.yaw_base + (fast_noise(self.timer * 0.1) * 0.8 - 0.4);
                    self.pause_timer = 0.5;
                }

                let yaw_diff = self.target_yaw - yaw;
                let yaw_step = yaw_diff * 2.0 * delta;
                let new_yaw = yaw + yaw_step;
                self.yaw_base = new_yaw;

                let sy = new_yaw.sin();
                let cy = new_yaw.cos();
                let tour_speed = self.speed * delta * 0.6;

                let mut new_pos = *pos;
                new_pos[0] -= sy * tour_speed;
                new_pos[2] -= cy * tour_speed;

                let ground_y = terrain::get_height(params, new_pos[0], new_pos[2]);
                let target_h = ground_y + 2.5;
                new_pos[1] += (target_h - new_pos[1]) * 3.0 * delta;

                let new_pitch = pitch + (-0.1 - pitch) * delta;

                Some(TourUpdate {
                    pos: new_pos,
                    yaw: new_yaw,
                    pitch: new_pitch,
                })
            }
            TourMode::None => None,
        }
    }

    pub fn any_input(&self, keys_mask: u32, joy_dx: f64, joy_dy: f64, gamepad_axes: &[f64; 4]) -> bool {
        if self.active == TourMode::None {
            return false;
        }
        let movement_keys = keys_mask & 0xFFFF;
        if movement_keys != 0 {
            return true;
        }
        if joy_dx.abs() > 0.3 || joy_dy.abs() > 0.3 {
            return true;
        }
        if gamepad_axes[0].abs() > 0.15 || gamepad_axes[1].abs() > 0.15
            || gamepad_axes[2].abs() > 0.15 || gamepad_axes[3].abs() > 0.15
        {
            return true;
        }
        false
    }
}

fn fast_noise(x: f64) -> f64 {
    let n = (x * 43758.5453).sin() * 0.5 + 0.5;
    n - n.floor()
}
