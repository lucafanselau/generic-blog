use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, EventTarget, HtmlCanvasElement, KeyboardEvent, MouseEvent, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
}

impl Key {
    pub const KEYS: [Key; 4] = [Key::W, Key::A, Key::S, Key::D];

    pub fn key_code(&self) -> u32 {
        // taken from https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
        match *self {
            Key::W => 87,
            Key::A => 65,
            Key::S => 83,
            Key::D => 68,
        }
    }
}

pub struct InputState {
    keys: Arc<RwLock<HashMap<Key, bool>>>,
    mouse_cbs: Arc<RwLock<Vec<Box<dyn Fn(i32, i32)>>>>,
    event_target: EventTarget,
    keydown_callback: Closure<dyn FnMut(KeyboardEvent)>,
    keyup_callback: Closure<dyn FnMut(KeyboardEvent)>,
    mouse_callback: Closure<dyn FnMut(MouseEvent)>,
}

impl InputState {
    pub fn register(document: Document, canvas: &HtmlCanvasElement) -> Self {
        let key_state = Arc::new(RwLock::new(HashMap::new()));
        let mouse_cbs: Arc<RwLock<Vec<Box<dyn Fn(i32, i32)>>>> = Default::default();

        let event_target: EventTarget = document.into();
        let keydown_callback = {
            let state = key_state.clone();
            Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if let Some((k, _c)) = Key::KEYS
                    .iter()
                    .map(|k| (k, k.key_code()))
                    .find(|(_, c)| event.key_code() == *c)
                {
                    let mut s = state
                        .write()
                        .expect("failed to get write access to input state");
                    s.insert(*k, true);
                }
            }) as Box<dyn FnMut(KeyboardEvent)>)
        };

        event_target
            .add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())
            .expect("failed to add keydown event");

        let keyup_callback = {
            let state = key_state.clone();
            Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if let Some((k, _c)) = Key::KEYS
                    .iter()
                    .map(|k| (k, k.key_code()))
                    .find(|(_, c)| event.key_code() == *c)
                {
                    let mut s = state
                        .write()
                        .expect("failed to get write access to input state");
                    s.insert(*k, false);
                }
            }) as Box<dyn FnMut(KeyboardEvent)>)
        };

        event_target
            .add_event_listener_with_callback("keyup", keyup_callback.as_ref().unchecked_ref())
            .expect("failed to add keyup event");

        let mouse_callback = {
            let cbs = mouse_cbs.clone();
            Closure::wrap(Box::new(move |event: MouseEvent| {
                for cb in cbs.read().unwrap().iter() {
                    cb(event.movement_x(), event.movement_y());
                }
            }) as Box<dyn FnMut(MouseEvent)>)
        };

        event_target
            .add_event_listener_with_callback("mousemove", mouse_callback.as_ref().unchecked_ref())
            .expect("failed to add mousemove event");

        Self {
            keys: key_state,
            mouse_cbs,
            event_target,
            keydown_callback,
            keyup_callback,
            mouse_callback,
        }
    }

    pub fn is_pressed(&self, key: &Key) -> bool {
        let k = self.keys.read().unwrap();
        *k.get(key).unwrap_or(&false)
    }

    pub fn add_mouse_cb<F: Fn(i32, i32) + 'static>(&self, cb: F) {
        let mut cbs = self.mouse_cbs.write().unwrap();
        cbs.push(Box::new(cb));
    }
}

impl Drop for InputState {
    fn drop(&mut self) {
        self.event_target
            .remove_event_listener_with_callback(
                "keydown",
                self.keydown_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove keydown handler");

        self.event_target
            .remove_event_listener_with_callback(
                "keyup",
                self.keyup_callback.as_ref().unchecked_ref(),
            )
            .expect("failed to remove keyup handler");
    }
}
