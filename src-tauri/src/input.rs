use crate::config::{AppConfig, KeyMap};
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse, Settings};
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

fn normalized_key_code(key: &str) -> Option<String> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    if key.len() == 1 {
        let char = key.chars().next()?.to_ascii_uppercase();
        if char.is_ascii_alphabetic() {
            return Some(format!("Key{char}"));
        }
        if char.is_ascii_digit() {
            return Some(format!("Digit{char}"));
        }
    }

    Some(key.to_string())
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use core_foundation::runloop::CFRunLoop;
    use core_graphics::{
        event::{
            CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
            CGEventType, CallbackResult, EventField, KeyCode,
        },
        event_source::{CGEventSource, CGEventSourceStateID},
    };
    use std::{
        collections::HashSet,
        sync::{Mutex, OnceLock},
        thread,
        time::{Duration, Instant},
    };

    static KEY_STATE: OnceLock<Mutex<KeyState>> = OnceLock::new();

    #[derive(Default)]
    struct KeyState {
        pressed: HashSet<u16>,
        left_down_at: Option<Instant>,
        dragging: bool,
    }

    pub fn start(config: Arc<RwLock<AppConfig>>, app: AppHandle) {
        KEY_STATE.get_or_init(|| Mutex::new(KeyState::default()));

        start_motion_loop(Arc::clone(&config));

        thread::spawn(move || {
            let events = vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
            ];

            let result = CGEventTap::with_enabled(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                events,
                move |_proxy, event_type, event| handle_event(event_type, event, &config, &app),
                CFRunLoop::run_current,
            );

            if result.is_err() {
                eprintln!("KeyPoint input hook failed: grant Accessibility/Input Monitoring permission to KeyPoint or the launching terminal.");
            }
        });
    }

    fn start_motion_loop(config: Arc<RwLock<AppConfig>>) {
        thread::spawn(move || loop {
            tick_motion(&config);
            thread::sleep(Duration::from_millis(16));
        });
    }

    fn tick_motion(config: &Arc<RwLock<AppConfig>>) {
        let current_config = match config.read() {
            Ok(config) => config.clone(),
            Err(_) => return,
        };

        let Some(state_lock) = KEY_STATE.get() else {
            return;
        };
        let Ok(mut state) = state_lock.lock() else {
            return;
        };

        if !current_config.mouse_mode_enabled {
            return;
        }

        let keymap = current_config.keymap.clone();
        let has_movement = has_movement_keys(&state.pressed, &keymap);
        if !has_movement {
            return;
        }

        if key_code_for(&keymap.scroll_modifier).is_some_and(|key| state.pressed.contains(&key)) {
            scroll_for_pressed_keys(&state.pressed, current_config.scroll_speed, &keymap);
            return;
        }

        if key_code_for(&keymap.left_click).is_some_and(|key| state.pressed.contains(&key))
            && !state.dragging
        {
            mouse_button(Direction::Press, Button::Left);
            state.dragging = true;
        }

        move_for_pressed_keys(&state.pressed, current_config.move_speed, &keymap);
    }

    fn handle_event(
        event_type: CGEventType,
        event: &CGEvent,
        config: &Arc<RwLock<AppConfig>>,
        app: &AppHandle,
    ) -> CallbackResult {
        if matches!(
            event_type,
            CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
        ) {
            return CallbackResult::Keep;
        }

        let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
        if matches!(event_type, CGEventType::FlagsChanged) {
            return CallbackResult::Keep;
        }

        let key_down = matches!(event_type, CGEventType::KeyDown);
        let key_up = matches!(event_type, CGEventType::KeyUp);

        if key_down {
            if let Some(result) = handle_key_down(keycode, event, config, app) {
                return result;
            }
        } else if key_up {
            if let Some(result) = handle_key_up(keycode, config) {
                return result;
            }
        }

        CallbackResult::Keep
    }

    fn handle_key_down(
        keycode: u16,
        event: &CGEvent,
        config: &Arc<RwLock<AppConfig>>,
        app: &AppHandle,
    ) -> Option<CallbackResult> {
        let current_config = config.read().ok()?.clone();
        let keymap = current_config.keymap.clone();

        if key_matches(keycode, &keymap.toggle_key) && has_toggle_modifiers(event, &keymap) {
            toggle_config(config, app);
            return Some(CallbackResult::Drop);
        }

        let state_lock = KEY_STATE.get()?;
        let mut state = state_lock.lock().ok()?;
        let was_pressed = state.pressed.contains(&keycode);
        state.pressed.insert(keycode);

        if !current_config.mouse_mode_enabled {
            return None;
        }

        if is_movement_key(keycode, &keymap) || key_matches(keycode, &keymap.scroll_modifier) {
            return Some(CallbackResult::Drop);
        }

        if key_matches(keycode, &keymap.left_click) {
            state.left_down_at.get_or_insert_with(Instant::now);
            return Some(CallbackResult::Drop);
        }

        if key_matches(keycode, &keymap.right_click) {
            if !was_pressed {
                mouse_button(Direction::Click, Button::Right);
            }
            return Some(CallbackResult::Drop);
        }

        None
    }

    fn handle_key_up(keycode: u16, config: &Arc<RwLock<AppConfig>>) -> Option<CallbackResult> {
        let state_lock = KEY_STATE.get()?;
        let mut state = state_lock.lock().ok()?;
        state.pressed.remove(&keycode);

        let current_config = config.read().ok()?.clone();
        let keymap = current_config.keymap;
        if !current_config.mouse_mode_enabled {
            if key_matches(keycode, &keymap.left_click) {
                state.left_down_at = None;
                state.dragging = false;
            }
            return None;
        }

        if key_matches(keycode, &keymap.left_click) {
            if state.dragging {
                mouse_button(Direction::Release, Button::Left);
            } else if state
                .left_down_at
                .map(|instant| instant.elapsed() <= Duration::from_millis(450))
                .unwrap_or(false)
            {
                mouse_button(Direction::Click, Button::Left);
            }
            state.left_down_at = None;
            state.dragging = false;
            return Some(CallbackResult::Drop);
        }

        if is_movement_key(keycode, &keymap)
            || key_matches(keycode, &keymap.scroll_modifier)
            || key_matches(keycode, &keymap.right_click)
        {
            return Some(CallbackResult::Drop);
        }

        None
    }

    fn has_toggle_modifiers(event: &CGEvent, keymap: &KeyMap) -> bool {
        let flags = event.get_flags();
        keymap
            .toggle_modifiers
            .iter()
            .all(|modifier| match modifier.as_str() {
                "Command" => flags.contains(core_graphics::event::CGEventFlags::CGEventFlagCommand),
                "Control" => flags.contains(core_graphics::event::CGEventFlags::CGEventFlagControl),
                "Shift" => flags.contains(core_graphics::event::CGEventFlags::CGEventFlagShift),
                "Alt" => flags.contains(core_graphics::event::CGEventFlags::CGEventFlagAlternate),
                _ => false,
            })
    }

    fn toggle_config(config: &Arc<RwLock<AppConfig>>, app: &AppHandle) {
        if let Ok(mut config) = config.write() {
            config.mouse_mode_enabled = !config.mouse_mode_enabled;
            let enabled = config.mouse_mode_enabled;
            let _ = crate::config::write_config(&config);
            reset_input_state();
            crate::update_mouse_mode_indicators(app, enabled);
        }
    }

    fn reset_input_state() {
        let Some(state_lock) = KEY_STATE.get() else {
            return;
        };
        if let Ok(mut state) = state_lock.lock() {
            if state.dragging {
                mouse_button(Direction::Release, Button::Left);
            }
            state.pressed.clear();
            state.left_down_at = None;
            state.dragging = false;
        }
    }

    fn move_for_pressed_keys(pressed: &HashSet<u16>, speed: i32, keymap: &KeyMap) {
        let step = movement_step(speed);
        let x = axis_value(pressed, &keymap.move_left, &keymap.move_right) * step;
        let y = axis_value(pressed, &keymap.move_up, &keymap.move_down) * step;

        if x == 0 && y == 0 {
            return;
        }

        with_enigo(|enigo| {
            let _ = enigo.move_mouse(x, y, Coordinate::Rel);
        });
    }

    fn scroll_for_pressed_keys(pressed: &HashSet<u16>, speed: i32, keymap: &KeyMap) {
        let horizontal = axis_value(pressed, &keymap.move_left, &keymap.move_right) * speed;
        let vertical = -axis_value(pressed, &keymap.move_up, &keymap.move_down) * speed;

        with_enigo(|enigo| {
            if vertical != 0 {
                let _ = enigo.scroll(vertical, Axis::Vertical);
            }
            if horizontal != 0 {
                let _ = enigo.scroll(horizontal, Axis::Horizontal);
            }
        });
    }

    fn axis_value(pressed: &HashSet<u16>, negative_key: &str, positive_key: &str) -> i32 {
        let negative = key_code_for(negative_key).is_some_and(|key| pressed.contains(&key)) as i32;
        let positive = key_code_for(positive_key).is_some_and(|key| pressed.contains(&key)) as i32;
        positive - negative
    }

    fn has_movement_keys(pressed: &HashSet<u16>, keymap: &KeyMap) -> bool {
        [
            &keymap.move_up,
            &keymap.move_down,
            &keymap.move_left,
            &keymap.move_right,
        ]
        .into_iter()
        .any(|key| key_code_for(key).is_some_and(|code| pressed.contains(&code)))
    }

    fn movement_step(speed: i32) -> i32 {
        (speed / 8).max(1)
    }

    fn is_movement_key(keycode: u16, keymap: &KeyMap) -> bool {
        key_matches(keycode, &keymap.move_up)
            || key_matches(keycode, &keymap.move_down)
            || key_matches(keycode, &keymap.move_left)
            || key_matches(keycode, &keymap.move_right)
    }

    fn key_matches(keycode: u16, key: &str) -> bool {
        key_code_for(key) == Some(keycode)
    }

    fn key_code_for(key: &str) -> Option<u16> {
        let key = normalized_key_code(key)?;
        Some(match key.as_str() {
            "KeyA" => KeyCode::ANSI_A,
            "KeyB" => KeyCode::ANSI_B,
            "KeyC" => KeyCode::ANSI_C,
            "KeyD" => KeyCode::ANSI_D,
            "KeyE" => KeyCode::ANSI_E,
            "KeyF" => KeyCode::ANSI_F,
            "KeyG" => KeyCode::ANSI_G,
            "KeyH" => KeyCode::ANSI_H,
            "KeyI" => KeyCode::ANSI_I,
            "KeyJ" => KeyCode::ANSI_J,
            "KeyK" => KeyCode::ANSI_K,
            "KeyL" => KeyCode::ANSI_L,
            "KeyM" => KeyCode::ANSI_M,
            "KeyN" => KeyCode::ANSI_N,
            "KeyO" => KeyCode::ANSI_O,
            "KeyP" => KeyCode::ANSI_P,
            "KeyQ" => KeyCode::ANSI_Q,
            "KeyR" => KeyCode::ANSI_R,
            "KeyS" => KeyCode::ANSI_S,
            "KeyT" => KeyCode::ANSI_T,
            "KeyU" => KeyCode::ANSI_U,
            "KeyV" => KeyCode::ANSI_V,
            "KeyW" => KeyCode::ANSI_W,
            "KeyX" => KeyCode::ANSI_X,
            "KeyY" => KeyCode::ANSI_Y,
            "KeyZ" => KeyCode::ANSI_Z,
            "Digit0" => KeyCode::ANSI_0,
            "Digit1" => KeyCode::ANSI_1,
            "Digit2" => KeyCode::ANSI_2,
            "Digit3" => KeyCode::ANSI_3,
            "Digit4" => KeyCode::ANSI_4,
            "Digit5" => KeyCode::ANSI_5,
            "Digit6" => KeyCode::ANSI_6,
            "Digit7" => KeyCode::ANSI_7,
            "Digit8" => KeyCode::ANSI_8,
            "Digit9" => KeyCode::ANSI_9,
            "Space" => KeyCode::SPACE,
            "Enter" => KeyCode::RETURN,
            "Tab" => KeyCode::TAB,
            "Escape" => KeyCode::ESCAPE,
            "Backspace" => KeyCode::DELETE,
            "Delete" => KeyCode::FORWARD_DELETE,
            "ArrowUp" => KeyCode::UP_ARROW,
            "ArrowDown" => KeyCode::DOWN_ARROW,
            "ArrowLeft" => KeyCode::LEFT_ARROW,
            "ArrowRight" => KeyCode::RIGHT_ARROW,
            "Home" => KeyCode::HOME,
            "End" => KeyCode::END,
            "PageUp" => KeyCode::PAGE_UP,
            "PageDown" => KeyCode::PAGE_DOWN,
            "Minus" => KeyCode::ANSI_MINUS,
            "Equal" => KeyCode::ANSI_EQUAL,
            "BracketLeft" => KeyCode::ANSI_LEFT_BRACKET,
            "BracketRight" => KeyCode::ANSI_RIGHT_BRACKET,
            "Backslash" => KeyCode::ANSI_BACKSLASH,
            "Semicolon" => KeyCode::ANSI_SEMICOLON,
            "Quote" => KeyCode::ANSI_QUOTE,
            "Comma" => KeyCode::ANSI_COMMA,
            "Period" => KeyCode::ANSI_PERIOD,
            "Slash" => KeyCode::ANSI_SLASH,
            "Backquote" => KeyCode::ANSI_GRAVE,
            "Numpad0" => KeyCode::ANSI_KEYPAD_0,
            "Numpad1" => KeyCode::ANSI_KEYPAD_1,
            "Numpad2" => KeyCode::ANSI_KEYPAD_2,
            "Numpad3" => KeyCode::ANSI_KEYPAD_3,
            "Numpad4" => KeyCode::ANSI_KEYPAD_4,
            "Numpad5" => KeyCode::ANSI_KEYPAD_5,
            "Numpad6" => KeyCode::ANSI_KEYPAD_6,
            "Numpad7" => KeyCode::ANSI_KEYPAD_7,
            "Numpad8" => KeyCode::ANSI_KEYPAD_8,
            "Numpad9" => KeyCode::ANSI_KEYPAD_9,
            "NumpadDecimal" => KeyCode::ANSI_KEYPAD_DECIMAL,
            "NumpadAdd" => KeyCode::ANSI_KEYPAD_PLUS,
            "NumpadSubtract" => KeyCode::ANSI_KEYPAD_MINUS,
            "NumpadMultiply" => KeyCode::ANSI_KEYPAD_MULTIPLY,
            "NumpadDivide" => KeyCode::ANSI_KEYPAD_DIVIDE,
            "NumpadEnter" => KeyCode::ANSI_KEYPAD_ENTER,
            "F1" => KeyCode::F1,
            "F2" => KeyCode::F2,
            "F3" => KeyCode::F3,
            "F4" => KeyCode::F4,
            "F5" => KeyCode::F5,
            "F6" => KeyCode::F6,
            "F7" => KeyCode::F7,
            "F8" => KeyCode::F8,
            "F9" => KeyCode::F9,
            "F10" => KeyCode::F10,
            "F11" => KeyCode::F11,
            "F12" => KeyCode::F12,
            _ => return None,
        })
    }

    fn mouse_button(direction: Direction, button: Button) {
        with_enigo(|enigo| {
            let _ = enigo.button(button, direction);
        });
    }

    fn with_enigo(action: impl FnOnce(&mut Enigo)) {
        if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
            action(&mut enigo);
        }
    }

    #[allow(dead_code)]
    fn _event_source() -> Option<CGEventSource> {
        CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::{
        collections::HashSet,
        ptr::{null, null_mut},
        sync::{Mutex, OnceLock},
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::GetAsyncKeyState,
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
                KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
                WM_SYSKEYUP,
            },
        },
    };

    const VK_SHIFT: u32 = 0x10;
    const VK_CONTROL: u32 = 0x11;
    const VK_ALT: u32 = 0x12;
    const VK_LSHIFT: u32 = 0xA0;
    const VK_RSHIFT: u32 = 0xA1;
    const VK_LCONTROL: u32 = 0xA2;
    const VK_RCONTROL: u32 = 0xA3;
    const VK_LALT: u32 = 0xA4;
    const VK_RALT: u32 = 0xA5;

    static KEY_STATE: OnceLock<Mutex<KeyState>> = OnceLock::new();
    static CONFIG: OnceLock<Arc<RwLock<AppConfig>>> = OnceLock::new();
    static APP: OnceLock<AppHandle> = OnceLock::new();

    #[derive(Default)]
    struct KeyState {
        pressed: HashSet<u32>,
        left_down_at: Option<Instant>,
        dragging: bool,
    }

    pub fn start(config: Arc<RwLock<AppConfig>>, app: AppHandle) {
        KEY_STATE.get_or_init(|| Mutex::new(KeyState::default()));
        let _ = CONFIG.set(Arc::clone(&config));
        let _ = APP.set(app);

        start_motion_loop(config);
        thread::spawn(move || unsafe {
            let module = GetModuleHandleW(null());
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
            if hook == null_mut() {
                eprintln!("KeyPoint input hook failed: SetWindowsHookExW returned null");
                return;
            }

            let mut message = std::mem::zeroed::<MSG>();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        });
    }

    fn start_motion_loop(config: Arc<RwLock<AppConfig>>) {
        thread::spawn(move || loop {
            tick_motion(&config);
            thread::sleep(Duration::from_millis(16));
        });
    }

    fn tick_motion(config: &Arc<RwLock<AppConfig>>) {
        let current_config = match config.read() {
            Ok(config) => config.clone(),
            Err(_) => return,
        };

        let Some(state_lock) = KEY_STATE.get() else {
            return;
        };
        let Ok(mut state) = state_lock.lock() else {
            return;
        };

        let keymap = current_config.keymap.clone();
        if !current_config.mouse_mode_enabled || !has_movement_keys(&state.pressed, &keymap) {
            return;
        }

        if vk_for(&keymap.scroll_modifier).is_some_and(|key| state.pressed.contains(&key)) {
            scroll_for_pressed_keys(&state.pressed, current_config.scroll_speed, &keymap);
            return;
        }

        if vk_for(&keymap.left_click).is_some_and(|key| state.pressed.contains(&key))
            && !state.dragging
        {
            mouse_button(Direction::Press, Button::Left);
            state.dragging = true;
        }

        move_for_pressed_keys(&state.pressed, current_config.move_speed, &keymap);
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code < 0 {
            return CallNextHookEx(null_mut(), code, wparam, lparam);
        }

        let event = *(lparam as *const KBDLLHOOKSTRUCT);
        let vk = event.vkCode;
        let message = wparam as u32;
        let suppress = match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => handle_key_down(vk),
            WM_KEYUP | WM_SYSKEYUP => handle_key_up(vk),
            _ => false,
        };

        if suppress {
            1
        } else {
            CallNextHookEx(null_mut(), code, wparam, lparam)
        }
    }

    fn handle_key_down(vk: u32) -> bool {
        let Some(config) = CONFIG.get() else {
            return false;
        };
        let current_config = match config.read() {
            Ok(config) => config.clone(),
            Err(_) => return false,
        };
        let keymap = current_config.keymap.clone();

        if key_matches(vk, &keymap.toggle_key) && has_toggle_modifiers_down(&keymap) {
            toggle_config();
            return true;
        }

        let Some(state_lock) = KEY_STATE.get() else {
            return false;
        };
        let Ok(mut state) = state_lock.lock() else {
            return false;
        };
        let was_pressed = state.pressed.contains(&vk);
        state.pressed.insert(vk);

        if !current_config.mouse_mode_enabled {
            return false;
        }

        if is_movement_key(vk, &keymap) || key_matches(vk, &keymap.scroll_modifier) {
            return true;
        }
        if key_matches(vk, &keymap.left_click) {
            state.left_down_at.get_or_insert_with(Instant::now);
            return true;
        }
        if key_matches(vk, &keymap.right_click) {
            if !was_pressed {
                mouse_button(Direction::Click, Button::Right);
            }
            return true;
        }

        false
    }

    fn handle_key_up(vk: u32) -> bool {
        let Some(state_lock) = KEY_STATE.get() else {
            return false;
        };
        let Ok(mut state) = state_lock.lock() else {
            return false;
        };
        state.pressed.remove(&vk);

        let Some(config) = CONFIG.get() else {
            return false;
        };
        let current_config = match config.read() {
            Ok(config) => config.clone(),
            Err(_) => return false,
        };

        let keymap = current_config.keymap;
        if !current_config.mouse_mode_enabled {
            if key_matches(vk, &keymap.left_click) {
                state.left_down_at = None;
                state.dragging = false;
            }
            return false;
        }

        if key_matches(vk, &keymap.left_click) {
            if state.dragging {
                mouse_button(Direction::Release, Button::Left);
            } else if state
                .left_down_at
                .map(|instant| instant.elapsed() <= Duration::from_millis(450))
                .unwrap_or(false)
            {
                mouse_button(Direction::Click, Button::Left);
            }
            state.left_down_at = None;
            state.dragging = false;
            return true;
        }

        is_movement_key(vk, &keymap)
            || key_matches(vk, &keymap.scroll_modifier)
            || key_matches(vk, &keymap.right_click)
    }

    fn toggle_config() {
        let Some(config) = CONFIG.get() else {
            return;
        };
        let Some(app) = APP.get() else {
            return;
        };

        if let Ok(mut config) = config.write() {
            config.mouse_mode_enabled = !config.mouse_mode_enabled;
            let enabled = config.mouse_mode_enabled;
            let _ = crate::config::write_config(&config);
            reset_input_state();
            crate::update_mouse_mode_indicators(app, enabled);
        }
    }

    fn reset_input_state() {
        let Some(state_lock) = KEY_STATE.get() else {
            return;
        };
        if let Ok(mut state) = state_lock.lock() {
            if state.dragging {
                mouse_button(Direction::Release, Button::Left);
            }
            state.pressed.clear();
            state.left_down_at = None;
            state.dragging = false;
        }
    }

    fn has_toggle_modifiers_down(keymap: &KeyMap) -> bool {
        unsafe {
            keymap
                .toggle_modifiers
                .iter()
                .all(|modifier| match modifier.as_str() {
                    "Control" => {
                        key_is_down(VK_CONTROL)
                            || key_is_down(VK_LCONTROL)
                            || key_is_down(VK_RCONTROL)
                    }
                    "Shift" => {
                        key_is_down(VK_SHIFT) || key_is_down(VK_LSHIFT) || key_is_down(VK_RSHIFT)
                    }
                    "Alt" => key_is_down(VK_ALT) || key_is_down(VK_LALT) || key_is_down(VK_RALT),
                    "Command" => false,
                    _ => false,
                })
        }
    }

    unsafe fn key_is_down(vk: u32) -> bool {
        (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0
    }

    fn move_for_pressed_keys(pressed: &HashSet<u32>, speed: i32, keymap: &KeyMap) {
        let step = movement_step(speed);
        let x = axis_value(pressed, &keymap.move_left, &keymap.move_right) * step;
        let y = axis_value(pressed, &keymap.move_up, &keymap.move_down) * step;

        if x == 0 && y == 0 {
            return;
        }

        with_enigo(|enigo| {
            let _ = enigo.move_mouse(x, y, Coordinate::Rel);
        });
    }

    fn scroll_for_pressed_keys(pressed: &HashSet<u32>, speed: i32, keymap: &KeyMap) {
        let horizontal = axis_value(pressed, &keymap.move_left, &keymap.move_right) * speed;
        let vertical = -axis_value(pressed, &keymap.move_up, &keymap.move_down) * speed;

        with_enigo(|enigo| {
            if vertical != 0 {
                let _ = enigo.scroll(vertical, Axis::Vertical);
            }
            if horizontal != 0 {
                let _ = enigo.scroll(horizontal, Axis::Horizontal);
            }
        });
    }

    fn axis_value(pressed: &HashSet<u32>, negative_key: &str, positive_key: &str) -> i32 {
        let negative = vk_for(negative_key).is_some_and(|key| pressed.contains(&key)) as i32;
        let positive = vk_for(positive_key).is_some_and(|key| pressed.contains(&key)) as i32;
        positive - negative
    }

    fn has_movement_keys(pressed: &HashSet<u32>, keymap: &KeyMap) -> bool {
        [
            &keymap.move_up,
            &keymap.move_down,
            &keymap.move_left,
            &keymap.move_right,
        ]
        .into_iter()
        .any(|key| vk_for(key).is_some_and(|code| pressed.contains(&code)))
    }

    fn movement_step(speed: i32) -> i32 {
        (speed / 8).max(1)
    }

    fn is_movement_key(vk: u32, keymap: &KeyMap) -> bool {
        key_matches(vk, &keymap.move_up)
            || key_matches(vk, &keymap.move_down)
            || key_matches(vk, &keymap.move_left)
            || key_matches(vk, &keymap.move_right)
    }

    fn key_matches(vk: u32, key: &str) -> bool {
        vk_for(key) == Some(vk)
    }

    fn vk_for(key: &str) -> Option<u32> {
        let key = normalized_key_code(key)?;
        if let Some(letter) = key.strip_prefix("Key") {
            let char = letter.chars().next()?;
            if letter.len() == 1 && char.is_ascii_uppercase() {
                return Some(char as u32);
            }
        }
        if let Some(digit) = key.strip_prefix("Digit") {
            let char = digit.chars().next()?;
            if digit.len() == 1 && char.is_ascii_digit() {
                return Some(char as u32);
            }
        }
        if let Some(digit) = key.strip_prefix("Numpad") {
            let char = digit.chars().next()?;
            if digit.len() == 1 && char.is_ascii_digit() {
                return Some(0x60 + char.to_digit(10)?);
            }
        }
        if let Some(function_key) = key.strip_prefix('F') {
            if let Ok(number) = function_key.parse::<u32>() {
                if (1..=12).contains(&number) {
                    return Some(0x70 + number - 1);
                }
            }
        }

        Some(match key.as_str() {
            "Space" => 0x20,
            "Enter" => 0x0D,
            "Tab" => 0x09,
            "Escape" => 0x1B,
            "Backspace" => 0x08,
            "Delete" => 0x2E,
            "ArrowUp" => 0x26,
            "ArrowDown" => 0x28,
            "ArrowLeft" => 0x25,
            "ArrowRight" => 0x27,
            "Home" => 0x24,
            "End" => 0x23,
            "PageUp" => 0x21,
            "PageDown" => 0x22,
            "Minus" => 0xBD,
            "Equal" => 0xBB,
            "BracketLeft" => 0xDB,
            "BracketRight" => 0xDD,
            "Backslash" => 0xDC,
            "Semicolon" => 0xBA,
            "Quote" => 0xDE,
            "Comma" => 0xBC,
            "Period" => 0xBE,
            "Slash" => 0xBF,
            "Backquote" => 0xC0,
            "NumpadDecimal" => 0x6E,
            "NumpadAdd" => 0x6B,
            "NumpadSubtract" => 0x6D,
            "NumpadMultiply" => 0x6A,
            "NumpadDivide" => 0x6F,
            "NumpadEnter" => 0x0D,
            _ => return None,
        })
    }

    fn mouse_button(direction: Direction, button: Button) {
        with_enigo(|enigo| {
            let _ = enigo.button(button, direction);
        });
    }

    fn with_enigo(action: impl FnOnce(&mut Enigo)) {
        if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
            action(&mut enigo);
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod other {
    use super::*;
    use std::{thread, time::Duration};

    pub fn start(_config: Arc<RwLock<AppConfig>>, _app: AppHandle) {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3600));
        });
    }
}

pub struct InputController;

impl InputController {
    pub fn start(config: Arc<RwLock<AppConfig>>, app: AppHandle) {
        #[cfg(target_os = "macos")]
        macos::start(config, app);

        #[cfg(target_os = "windows")]
        windows::start(config, app);

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        other::start(config, app);
    }
}
