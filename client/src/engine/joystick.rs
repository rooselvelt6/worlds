use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const BASE_RADIUS: f64 = 45.0;
const KNOB_RADIUS: f64 = 18.0;

pub struct Joystick {
    canvas: web_sys::HtmlCanvasElement,
    ctx: web_sys::CanvasRenderingContext2d,
    active: Rc<Cell<bool>>,
    dx: Rc<Cell<f64>>,
    dy: Rc<Cell<f64>>,
    _touch_start: Option<Closure<dyn Fn(web_sys::TouchEvent)>>,
    _touch_move: Option<Closure<dyn Fn(web_sys::TouchEvent)>>,
    _touch_end: Option<Closure<dyn Fn(web_sys::TouchEvent)>>,
    _pointer_down: Option<Closure<dyn Fn(web_sys::PointerEvent)>>,
    _pointer_move: Option<Closure<dyn Fn(web_sys::PointerEvent)>>,
    _pointer_up: Option<Closure<dyn Fn(web_sys::PointerEvent)>>,
    anim_id: Option<i32>,
}

impl Joystick {
    pub fn new(canvas: web_sys::HtmlCanvasElement, dx: Rc<Cell<f64>>, dy: Rc<Cell<f64>>) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "no 2d context")?
            .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
            .ok_or("failed to get 2d context")?;

        let active = Rc::new(Cell::new(false));
        let pointer_id = Rc::new(Cell::new(0));

        let el: &web_sys::Element = canvas.as_ref();
        let el2 = canvas.clone();

        let active_c = active.clone();
        let dx_c = dx.clone();
        let dy_c = dy.clone();
        let touch_start = Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |e: web_sys::TouchEvent| {
            if let Some(touch) = e.touches().get(0) {
                let rect = el2.get_bounding_client_rect();
                let cx = rect.width() / 2.0;
                let cy = rect.height() / 2.0;
                dx_c.set((touch.client_x() as f64 - rect.left() - cx) / BASE_RADIUS);
                dy_c.set((touch.client_y() as f64 - rect.top() - cy) / BASE_RADIUS);
                active_c.set(true);
            }
            e.prevent_default();
        });

        let el3 = canvas.clone();
        let _active_c2 = active.clone();
        let dx_c2 = dx.clone();
        let dy_c2 = dy.clone();
        let touch_move = Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |e: web_sys::TouchEvent| {
            if let Some(touch) = e.touches().get(0) {
                let rect = el3.get_bounding_client_rect();
                let cx = rect.width() / 2.0;
                let cy = rect.height() / 2.0;
                dx_c2.set((touch.client_x() as f64 - rect.left() - cx) / BASE_RADIUS);
                dy_c2.set((touch.client_y() as f64 - rect.top() - cy) / BASE_RADIUS);
            }
            e.prevent_default();
        });

        let active_c3 = active.clone();
        let dx_c3 = dx.clone();
        let dy_c3 = dy.clone();
        let touch_end = Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |_: web_sys::TouchEvent| {
            dx_c3.set(0.0);
            dy_c3.set(0.0);
            active_c3.set(false);
        });

        let active_c4 = active.clone();
        let dx_c4 = dx.clone();
        let dy_c4 = dy.clone();
        let pid_c = pointer_id.clone();
        let canvas4 = canvas.clone();
        let pointer_down = Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
            let rect = canvas4.get_bounding_client_rect();
            let cx = rect.width() / 2.0;
            let cy = rect.height() / 2.0;
            dx_c4.set((e.client_x() as f64 - rect.left() - cx) / BASE_RADIUS);
            dy_c4.set((e.client_y() as f64 - rect.top() - cy) / BASE_RADIUS);
            active_c4.set(true);
            pid_c.set(e.pointer_id());
            let _ = canvas4.set_pointer_capture(e.pointer_id());
        });

        let dx_c5 = dx.clone();
        let dy_c5 = dy.clone();
        let canvas5 = canvas.clone();
        let pointer_move = Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
            let rect = canvas5.get_bounding_client_rect();
            let cx = rect.width() / 2.0;
            let cy = rect.height() / 2.0;
            dx_c5.set((e.client_x() as f64 - rect.left() - cx) / BASE_RADIUS);
            dy_c5.set((e.client_y() as f64 - rect.top() - cy) / BASE_RADIUS);
        });

        let canvas6 = canvas.clone();
        let active_c6 = active.clone();
        let dx_c6 = dx.clone();
        let dy_c6 = dy.clone();
        let pid_c2 = pointer_id.clone();
        let pointer_up = Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |_: web_sys::PointerEvent| {
            dx_c6.set(0.0);
            dy_c6.set(0.0);
            active_c6.set(false);
            let el: &web_sys::Element = canvas6.as_ref();
            let _ = el.release_pointer_capture(pid_c2.get());
        });

        el.add_event_listener_with_callback("touchstart", touch_start.as_ref().unchecked_ref()).ok();
        el.add_event_listener_with_callback("touchmove", touch_move.as_ref().unchecked_ref()).ok();
        el.add_event_listener_with_callback("touchend", touch_end.as_ref().unchecked_ref()).ok();
        el.add_event_listener_with_callback("pointerdown", pointer_down.as_ref().unchecked_ref()).ok();
        let doc = web_sys::window().unwrap().document().unwrap();
        doc.add_event_listener_with_callback("pointermove", pointer_move.as_ref().unchecked_ref()).ok();
        doc.add_event_listener_with_callback("pointerup", pointer_up.as_ref().unchecked_ref()).ok();

        Ok(Self {
            canvas,
            ctx,
            active,
            dx,
            dy,
            _touch_start: Some(touch_start),
            _touch_move: Some(touch_move),
            _touch_end: Some(touch_end),
            _pointer_down: Some(pointer_down),
            _pointer_move: Some(pointer_move),
            _pointer_up: Some(pointer_up),
            anim_id: None,
        })
    }

    pub fn start_render(&mut self) {
        let canvas = self.canvas.clone();
        let ctx = self.ctx.clone();
        let active = self.active.clone();
        let dx = self.dx.clone();
        let dy = self.dy.clone();

        let closure = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let closure2 = closure.clone();

        let cb = Closure::<dyn FnMut()>::new(move || {
            let w = canvas.width() as f64;
            let h = canvas.height() as f64;
            let cx = w / 2.0;
            let cy = h / 2.0;

            ctx.clear_rect(0.0, 0.0, w, h);

            let a = active.get();
            let ddx = dx.get().clamp(-1.0, 1.0);
            let ddy = dy.get().clamp(-1.0, 1.0);

            ctx.set_global_alpha(if a { 0.5 } else { 0.25 });
            ctx.begin_path();
            ctx.arc(cx, cy, BASE_RADIUS, 0.0, std::f64::consts::TAU).ok();
            ctx.set_fill_style_str("rgba(255,255,255,0.08)");
            ctx.fill();
            ctx.set_stroke_style_str("rgba(255,255,255,0.15)");
            ctx.set_line_width(1.5);
            ctx.stroke();

            let kx = cx + ddx * BASE_RADIUS * 0.6;
            let ky = cy + ddy * BASE_RADIUS * 0.6;
            ctx.set_global_alpha(if a { 0.8 } else { 0.4 });
            ctx.begin_path();
            ctx.arc(kx, ky, KNOB_RADIUS, 0.0, std::f64::consts::TAU).ok();
            ctx.set_fill_style_str("rgba(255,255,255,0.15)");
            ctx.fill();
            ctx.set_stroke_style_str("rgba(255,255,255,0.3)");
            ctx.set_line_width(1.0);
            ctx.stroke();

            if let Some(ref c) = *closure2.borrow() {
                web_sys::window().unwrap()
                    .request_animation_frame(c.as_ref().unchecked_ref()).ok();
            }
        });

        *closure.borrow_mut() = Some(cb);

        let id = {
            let borrow = closure.borrow();
            borrow.as_ref().and_then(|c| {
                web_sys::window().unwrap()
                    .request_animation_frame(c.as_ref().unchecked_ref()).ok()
            })
        };
        self.anim_id = id;
    }

    pub fn get_vector(&self) -> (f64, f64) {
        (self.dx.get().clamp(-1.0, 1.0), self.dy.get().clamp(-1.0, 1.0))
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
    }
}

impl Drop for Joystick {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
    }
}
