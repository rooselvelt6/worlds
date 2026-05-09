use std::cell::Cell;
use std::rc::Rc;

pub struct Camera {
    pub pos: [f64; 3],
    pub yaw: Rc<Cell<f64>>,
    pub pitch: Rc<Cell<f64>>,
}

impl Camera {
    pub fn new(yaw: Rc<Cell<f64>>, pitch: Rc<Cell<f64>>) -> Self {
        Self {
            pos: [50.0, 25.0, 50.0],
            yaw,
            pitch,
        }
    }
}
