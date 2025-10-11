
use resonance_core::math::Vec2;
use std::collections::HashSet;
use winit::event::MouseButton;

#[derive(Debug)]
pub struct MouseState {
    position: Vec2,
    delta: Vec2,
    pressed: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
    scroll_delta: f32,
}

impl MouseState {

    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            scroll_delta: 0.0,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.delta = position - self.position;
        self.position = position;
    }

    pub fn update_position(&mut self, x: f32, y: f32) {
        let new_position = Vec2::new(x, y);
        self.delta = new_position - self.position;
        self.position = new_position;
    }

    pub fn press_button(&mut self, button: MouseButton) {
        if self.pressed.insert(button) {
            self.just_pressed.insert(button);
        }
    }

    pub fn release_button(&mut self, button: MouseButton) {
        if self.pressed.remove(&button) {
            self.just_released.insert(button);
        }
    }

    pub fn scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    pub fn add_motion_delta(&mut self, dx: f32, dy: f32) {
        self.delta.x += dx;
        self.delta.y += dy;
    }

    pub fn update(&mut self) {
        self.delta = Vec2::ZERO;
        self.just_pressed.clear();
        self.just_released.clear();
        self.scroll_delta = 0.0;
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}