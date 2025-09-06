use std::collections::HashSet;
use glam::Vec2;
use winit::event::{WindowEvent, ElementState, KeyEvent, Touch, TouchPhase};
use winit::keyboard::{PhysicalKey};

use gilrs::{Gilrs, Button, Axis};

use crate::{Action, Axis2, InputContext};
use crate::bindings::{BindingSet, GamepadButton};

pub struct InputManager {
    pub context: InputContext,
    pub bindings: BindingSet,

    // pressed / just-pressed states
    pressed: HashSet<Action>,
    just_pressed: HashSet<Action>,

    // axes
    pub move_axis: Axis2,
    pub look_axis: Axis2,

    // mouse capture / sensitivity
    pub look_sensitivity: f32,

    // gamepad
    gilrs: Option<Gilrs>,

    // touch (virtual joystick)
    touch_active: bool,
    touch_id: Option<u64>,
    touch_origin: Option<Vec2>,
    touch_current: Option<Vec2>,
}

impl InputManager {
    pub fn new(context: InputContext, bindings: BindingSet) -> Self {
        let gilrs = Gilrs::new().ok();
        Self {
            context, bindings,
            pressed: HashSet::new(), just_pressed: HashSet::new(),
            move_axis: Axis2::default(), look_axis: Axis2::default(),
            look_sensitivity: 0.12,
            gilrs,
            touch_active:false, touch_id:None, touch_origin:None, touch_current:None,
        }
    }

    pub fn set_context(&mut self, cx: InputContext) { self.context = cx; }

    #[inline] pub fn is_down(&self, a: Action) -> bool { self.pressed.contains(&a) }
    #[inline] pub fn just_pressed(&self, a: Action) -> bool { self.just_pressed.contains(&a) }

    pub fn clear_frame(&mut self) { self.just_pressed.clear(); }

    pub fn process_window_event(&mut self, ev: &WindowEvent) {
        match ev {
            WindowEvent::KeyboardInput { event: KeyEvent{ state, physical_key: PhysicalKey::Code(code), .. }, .. } => {
                let actions: Vec<_> = self.bindings.actions.iter()
                    .filter_map(|(action, b)| {
                        if b.key == Some(*code) {
                            Some(*action)
                        } else {
                            None
                        }
                    })
                    .collect();
                
                for action in actions {
                    self.set_action(action, *state == ElementState::Pressed);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let actions: Vec<_> = self.bindings.actions.iter()
                    .filter_map(|(action, b)| {
                        if b.mouse == Some(*button) {
                            Some(*action)
                        } else {
                            None
                        }
                    })
                    .collect();
                
                for action in actions {
                    self.set_action(action, *state == ElementState::Pressed);
                }
            }
            WindowEvent::Touch(Touch { phase, id, location, .. }) => {
                match phase {
                    TouchPhase::Started => {
                        self.touch_active = true;
                        self.touch_id = Some(*id);
                        self.touch_origin = Some(glam::vec2(location.x as f32, location.y as f32));
                        self.touch_current = self.touch_origin;
                    }
                    TouchPhase::Moved => {
                        if self.touch_active && self.touch_id == Some(*id) {
                            self.touch_current = Some(glam::vec2(location.x as f32, location.y as f32));
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        if self.touch_id == Some(*id) {
                            self.touch_active = false;
                            self.touch_id = None;
                            self.touch_origin = None;
                            self.touch_current = None;
                            self.move_axis = Axis2::default();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn poll_gamepads(&mut self) {
        // Collect events first to avoid borrowing conflicts
        let mut events = Vec::new();
        if let Some(g) = self.gilrs.as_mut() {
            while let Some(ev) = g.next_event() {
                events.push(ev);
            }
        }
        
        // Process events after collecting them
        for ev in events {
            use gilrs::EventType::*;
            match ev.event {
                ButtonPressed(b, _) => self.handle_button(b, true),
                ButtonReleased(b, _) => self.handle_button(b, false),
                AxisChanged(a, v, _) => self.handle_axis(a, v),
                _ => {}
            }
        }
        
        // Virtual joystick from touch:
        if let (Some(o), Some(c)) = (self.touch_origin, self.touch_current) {
            let delta = (c - o) / 80.0; // pixels to normalized
            self.move_axis.x = delta.x.clamp(-1.0, 1.0);
            self.move_axis.y = (-delta.y).clamp(-1.0, 1.0);
        }
    }

    fn handle_button(&mut self, b: Button, down: bool) {
        use Button::*;
        let map = |b: Button| -> Option<crate::bindings::GamepadButton> {
            Some(match b {
                South => GamepadButton::South,
                East => GamepadButton::East,
                West => GamepadButton::West,
                North => GamepadButton::North,
                LeftTrigger => GamepadButton::L2,
                RightTrigger => GamepadButton::R2,
                LeftTrigger2 => GamepadButton::L1,
                RightTrigger2 => GamepadButton::R1,
                Select => GamepadButton::Select,
                Start => GamepadButton::Start,
                LeftThumb => GamepadButton::LStick,
                RightThumb => GamepadButton::RStick,
                DPadUp => GamepadButton::DPadUp,
                DPadDown => GamepadButton::DPadDown,
                DPadLeft => GamepadButton::DPadLeft,
                DPadRight => GamepadButton::DPadRight,
                _ => return None,
            })
        };
        if let Some(gb) = map(b) {
            let actions: Vec<_> = self.bindings.actions.iter()
                .filter_map(|(action, bind)| {
                    if bind.gamepad == Some(gb) {
                        Some(*action)
                    } else {
                        None
                    }
                })
                .collect();
                
            for action in actions {
                self.set_action(action, down);
            }
        }
    }

    fn handle_axis(&mut self, a: Axis, val: f32) {
        use Axis::*;
        let apply = |bind: &crate::bindings::AxisBinding, v: f32| -> f32 {
            let mut t = v;
            if bind.invert { t = -t; }
            if t.abs() < bind.deadzone { 0.0 } else { t }
        };
        let (mx,my) = &self.bindings.move_axes;
        let (lx,ly) = &self.bindings.look_axes;

        match a {
            LeftStickX => self.move_axis.x = apply(mx, val),
            LeftStickY => self.move_axis.y = apply(my, val),
            RightStickX => self.look_axis.x = apply(lx, val),
            RightStickY => self.look_axis.y = apply(ly, val),
            _ => {}
        }
    }

    fn set_action(&mut self, a: Action, down: bool) {
        if down {
            if !self.pressed.contains(&a) {
                self.just_pressed.insert(a);
            }
            self.pressed.insert(a);
        } else {
            self.pressed.remove(&a);
        }
    }
}
