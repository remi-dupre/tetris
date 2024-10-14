use bevy::input::keyboard::KeyboardInput;
use bevy::input::touch::TouchPhase;
use bevy::input::ButtonState;
use bevy::prelude::*;

use super::resources::*;

pub(crate) fn collect_keyboard_presses(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut player_input_queue: ResMut<PlayerInputQueue>,
) {
    for event in keyboard_input_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.key_code {
            KeyCode::ArrowLeft => player_input_queue.push_back(PlayerInput::MoveLeft),
            KeyCode::ArrowRight => player_input_queue.push_back(PlayerInput::MoveRight),
            KeyCode::ArrowUp | KeyCode::KeyX => {
                player_input_queue.push_back(PlayerInput::RotateRight)
            }
            KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::KeyZ => {
                player_input_queue.push_back(PlayerInput::RotateLeft)
            }
            KeyCode::Space => player_input_queue.push_back(PlayerInput::HardDrop),
            _ => {}
        }
    }
}

pub(crate) fn debug_touchscreen(mut touch_events: EventReader<TouchInput>) {
    for event in touch_events.read() {
        info!("{event:?}")
    }
}

pub(crate) fn touch_start(
    mut touch_events: EventReader<TouchInput>,
    mut touch_state: ResMut<TouchStateRegistry>,
) {
    for event in touch_events.read() {
        if event.phase == TouchPhase::Started {
            touch_state.touch_start.insert(
                event.id,
                TouchState {
                    start_position: event.position,
                    drawn_input: None,
                },
            );
        }
    }
}

pub(crate) fn touch_end(
    mut touch_events: EventReader<TouchInput>,
    mut touch_state: ResMut<TouchStateRegistry>,
) {
    for event in touch_events.read() {
        if matches!(event.phase, TouchPhase::Ended | TouchPhase::Canceled) {
            touch_state.touch_start.remove(&event.id);
        }
    }
}

pub(crate) fn collect_touch_moves(
    mut touch_events: EventReader<TouchInput>,
    mut player_input_queue: ResMut<PlayerInputQueue>,
    mut touch_states: ResMut<TouchStateRegistry>,
) {
    for event in touch_events.read() {
        let Some(touch_state) = touch_states.touch_start.get_mut(&event.id) else {
            continue;
        };

        if touch_state.drawn_input.is_some() {
            continue;
        }

        let delta = event.position - touch_state.start_position;

        if delta.length() < 50.0 {
            continue;
        }

        if delta.x > delta.y.abs() {
            touch_state.drawn_input = Some(PlayerInput::MoveRight);
            player_input_queue.push_back(PlayerInput::MoveRight);
        } else if delta.x < -delta.y.abs() {
            touch_state.drawn_input = Some(PlayerInput::MoveLeft);
            player_input_queue.push_back(PlayerInput::MoveLeft);
        } else if delta.y < -delta.x.abs() {
            touch_state.drawn_input = Some(PlayerInput::RotateRight);
            player_input_queue.push_back(PlayerInput::RotateLeft);
        } else if delta.y > delta.x.abs() {
            touch_state.drawn_input = Some(PlayerInput::HardDrop);
            player_input_queue.push_back(PlayerInput::HardDrop);
        }
    }
}
