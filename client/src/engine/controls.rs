use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlCanvasElement, KeyboardEvent, MouseEvent};

pub struct Controls {
    pub keys: Rc<Cell<u16>>,
    pub mouse_enabled: Rc<Cell<bool>>,
    pub yaw: Rc<Cell<f64>>,
    pub pitch: Rc<Cell<f64>>,
    pub sensitivity: Rc<Cell<f64>>,
    _kbd_down: Option<Closure<dyn Fn(KeyboardEvent)>>,
    _kbd_up: Option<Closure<dyn Fn(KeyboardEvent)>>,
    _mouse_move: Option<Closure<dyn Fn(MouseEvent)>>,
    _touch: Vec<Closure<dyn Fn(MouseEvent)>>,
    _pointer_change: Option<Closure<dyn Fn(web_sys::Event)>>,
    _click: Option<Closure<dyn Fn(MouseEvent)>>,
}

pub const MASK_W: u16 = 1 << 0;
pub const MASK_A: u16 = 1 << 1;
pub const MASK_S: u16 = 1 << 2;
pub const MASK_D: u16 = 1 << 3;
pub const MASK_SPACE: u16 = 1 << 4;
pub const MASK_SHIFT: u16 = 1 << 5;
pub const MASK_Q: u16 = 1 << 6;
pub const MASK_E: u16 = 1 << 7;
pub const MASK_C: u16 = 1 << 8;
pub const MASK_F12: u16 = 1 << 9;

impl Controls {
    pub fn new(yaw: Rc<Cell<f64>>, pitch: Rc<Cell<f64>>) -> Self {
        Self {
            keys: Rc::new(Cell::new(0)),
            mouse_enabled: Rc::new(Cell::new(false)),
            yaw,
            pitch,
            sensitivity: Rc::new(Cell::new(1.0)),
            _kbd_down: None,
            _kbd_up: None,
            _mouse_move: None,
            _touch: Vec::new(),
            _pointer_change: None,
            _click: None,
        }
    }

    pub fn attach(&mut self, canvas: &HtmlCanvasElement) {
        let doc = web_sys::window().unwrap().document().unwrap();
        let keys = self.keys.clone();
        let kbd_down = Closure::<dyn Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            let mut k = keys.get();
            match e.key().as_str() {
                "w" | "W" | "ArrowUp" => k |= MASK_W,
                "s" | "S" | "ArrowDown" => k |= MASK_S,
                "a" | "A" | "ArrowLeft" => k |= MASK_A,
                "d" | "D" | "ArrowRight" => k |= MASK_D,
                " " => k |= MASK_SPACE,
                "Shift" => k |= MASK_SHIFT,
                "q" | "Q" => k |= MASK_Q,
                "e" | "E" => k |= MASK_E,
                "c" | "C" => k |= MASK_C,
                "F12" => k |= MASK_F12,
                _ => {}
            }
            keys.set(k);
            e.prevent_default();
        });
        doc.add_event_listener_with_callback("keydown", kbd_down.as_ref().unchecked_ref()).ok();

        let keys2 = self.keys.clone();
        let kbd_up = Closure::<dyn Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            let mut k = keys2.get();
            match e.key().as_str() {
                "w" | "W" | "ArrowUp" => k &= !MASK_W,
                "s" | "S" | "ArrowDown" => k &= !MASK_S,
                "a" | "A" | "ArrowLeft" => k &= !MASK_A,
                "d" | "D" | "ArrowRight" => k &= !MASK_D,
                " " => k &= !MASK_SPACE,
                "Shift" => k &= !MASK_SHIFT,
                "q" | "Q" => k &= !MASK_Q,
                "e" | "E" => k &= !MASK_E,
                "c" | "C" => k &= !MASK_C,
                "F12" => k &= !MASK_F12,
                _ => {}
            }
            keys2.set(k);
        });
        doc.add_event_listener_with_callback("keyup", kbd_up.as_ref().unchecked_ref()).ok();

        let yaw_c = self.yaw.clone();
        let pitch_c = self.pitch.clone();
        let me = self.mouse_enabled.clone();
        let sens = self.sensitivity.clone();
        let mouse_move = Closure::<dyn Fn(MouseEvent)>::new(move |e: MouseEvent| {
            if me.get() {
                let s = sens.get();
                yaw_c.set(yaw_c.get() - e.movement_x() as f64 * 0.002 * s);
                let p = (pitch_c.get() - e.movement_y() as f64 * 0.002 * s).max(-1.5).min(1.5);
                pitch_c.set(p);
            }
        });
        doc.add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref()).ok();

        let me2 = self.mouse_enabled.clone();
        let pc = Closure::<dyn Fn(web_sys::Event)>::new(move |_: web_sys::Event| {
            let doc2 = web_sys::window().unwrap().document().unwrap();
            me2.set(doc2.pointer_lock_element().is_some());
        });
        doc.add_event_listener_with_callback("pointerlockchange", pc.as_ref().unchecked_ref()).ok();

        let el: &Element = canvas.as_ref();
        let canvas2 = canvas.clone();
        let click = Closure::<dyn Fn(MouseEvent)>::new(move |_: MouseEvent| {
            let el2: &Element = canvas2.as_ref();
            let _ = el2.request_pointer_lock();
        });
        el.add_event_listener_with_callback("click", click.as_ref().unchecked_ref()).ok();

        self._kbd_down = Some(kbd_down);
        self._kbd_up = Some(kbd_up);
        self._mouse_move = Some(mouse_move);
        self._pointer_change = Some(pc);
        self._click = Some(click);

        canvas.focus().ok();
    }

    pub fn set_sensitivity(&self, val: f64) {
        self.sensitivity.set(val);
    }
}
