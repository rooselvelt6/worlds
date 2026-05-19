use wasm_bindgen::JsCast;

#[derive(Clone, Default)]
pub struct GamepadState {
    pub connected: bool,
    pub axes: [f64; 4],
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub lb: bool,
    pub rb: bool,
    pub lt: f64,
    pub rt: f64,
    pub left_stick: bool,
    pub right_stick: bool,
    pub start: bool,
    pub select: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
}

pub fn poll_gamepad() -> GamepadState {
    let nav = match web_sys::window() {
        Some(w) => w.navigator(),
        None => return GamepadState::default(),
    };
    let arr = match nav.get_gamepads() {
        Ok(a) => a,
        Err(_) => return GamepadState::default(),
    };
    let len = arr.length();
    if len == 0 {
        return GamepadState::default();
    }
    let first = arr.get(0);
    if first.is_null() || first.is_undefined() {
        return GamepadState::default();
    }
    let gp: web_sys::Gamepad = match first.dyn_into() {
        Ok(g) => g,
        Err(_) => return GamepadState::default(),
    };
    if !gp.connected() {
        return GamepadState::default();
    }

    let raw_axes: js_sys::Array = gp.axes().dyn_into().unwrap_or_else(|_| js_sys::Array::new());
    let mut axes = [0.0_f64; 4];
    for i in 0..4u32 {
        axes[i as usize] = raw_axes.get(i).as_f64().unwrap_or(0.0);
    }

    let raw_btns: js_sys::Array = gp.buttons().dyn_into().unwrap_or_else(|_| js_sys::Array::new());
    let btn_len = raw_btns.length() as usize;
    let btn = |i: u32| -> Option<(bool, f64)> {
        let idx = i as usize;
        if idx >= btn_len { return None; }
        let v = raw_btns.get(i);
        if v.is_null() || v.is_undefined() { return None; }
        let b: web_sys::GamepadButton = v.dyn_into().ok()?;
        Some((b.pressed(), b.value()))
    };
    let b = |i: u32| { btn(i).map(|(p, _)| p).unwrap_or(false) };
    let bv = |i: u32| { btn(i).map(|(_, v)| v).unwrap_or(0.0) };

    GamepadState {
        connected: true,
        axes: [
            deadzone(axes[0]),
            deadzone(axes[1]),
            deadzone(axes[2]),
            deadzone(axes[3]),
        ],
        a: b(0),
        b: b(1),
        x: b(2),
        y: b(3),
        lb: b(4),
        rb: b(5),
        lt: bv(6),
        rt: bv(7),
        select: b(8),
        start: b(9),
        left_stick: b(10),
        right_stick: b(11),
        dpad_up: b(12),
        dpad_down: b(13),
        dpad_left: b(14),
        dpad_right: b(15),
    }
}

fn deadzone(v: f64) -> f64 {
    if v.abs() < 0.15 { 0.0 } else { v }
}
