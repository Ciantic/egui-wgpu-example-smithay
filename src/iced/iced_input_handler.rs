use iced::Point;
use iced::event::Event as IcedEvent;
use iced::keyboard::Key;
use iced::keyboard::Location;
use iced::keyboard::Modifiers as IcedModifiers;
use iced::keyboard::key::Named;
use iced::keyboard::key::NativeCode;
use iced::keyboard::key::Physical;
use iced::mouse::Button as IcedMouseButton;
use iced::mouse::ScrollDelta;
use log::trace;
use smithay_client_toolkit::seat::keyboard::KeyEvent;
use smithay_client_toolkit::seat::keyboard::Keysym;
use smithay_client_toolkit::seat::keyboard::Modifiers as WaylandModifiers;
use smithay_client_toolkit::seat::pointer::PointerEvent;
use smithay_client_toolkit::seat::pointer::PointerEventKind;
use smithay_clipboard::Clipboard;
use smol_str::SmolStr;
use std::time::Instant;

/// Handles input events from Wayland and converts them to Iced events
pub struct WaylandToIcedInput {
    modifiers: IcedModifiers,
    pointer_pos: (f64, f64),
    events: Vec<IcedEvent>,
    screen_width: u32,
    screen_height: u32,
    start_time: Instant,
    clipboard: Clipboard,
    last_key_utf8: Option<String>,
}

impl WaylandToIcedInput {
    pub fn new(clipboard: Clipboard) -> Self {
        Self {
            modifiers: IcedModifiers::default(),
            pointer_pos: (0.0, 0.0),
            events: Vec::new(),
            screen_width: 256,
            screen_height: 256,
            start_time: Instant::now(),
            clipboard,
            last_key_utf8: None,
        }
    }

    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn set_pointer_position(&mut self, x: f64, y: f64) {
        self.pointer_pos = (x, y);
    }

    pub fn handle_pointer_event(&mut self, event: &PointerEvent) {
        trace!("[INPUT] Pointer event: {:?}", event.kind);
        match &event.kind {
            PointerEventKind::Enter { .. } => {
                trace!("[INPUT] Pointer entered surface");
                let (x, y) = event.position;
                self.pointer_pos = (x, y);
                self.events
                    .push(IcedEvent::Mouse(iced::mouse::Event::CursorEntered));
            }
            PointerEventKind::Leave { .. } => {
                trace!("[INPUT] Pointer left surface");
                self.events
                    .push(IcedEvent::Mouse(iced::mouse::Event::CursorLeft));
            }
            PointerEventKind::Motion { .. } => {
                let (x, y) = event.position;
                self.pointer_pos = (x, y);
                trace!("[INPUT] Pointer moved to: ({}, {})", x, y);
                self.events
                    .push(IcedEvent::Mouse(iced::mouse::Event::CursorMoved {
                        position: Point::new(x as f32, y as f32),
                    }));
            }
            PointerEventKind::Press { button, .. } => {
                trace!("[INPUT] Pointer button pressed: {}", button);
                if let Some(iced_button) = wayland_button_to_iced(*button) {
                    trace!("[INPUT] Mapped to Iced button: {:?}", iced_button);
                    self.events
                        .push(IcedEvent::Mouse(iced::mouse::Event::ButtonPressed(
                            iced_button,
                        )));
                }
            }
            PointerEventKind::Release { button, .. } => {
                trace!("[INPUT] Pointer button released: {}", button);
                if let Some(iced_button) = wayland_button_to_iced(*button) {
                    self.events
                        .push(IcedEvent::Mouse(iced::mouse::Event::ButtonReleased(
                            iced_button,
                        )));
                }
            }
            PointerEventKind::Axis {
                horizontal,
                vertical,
                ..
            } => {
                // Handle scroll events
                let scroll_delta = ScrollDelta::Lines {
                    x: horizontal.discrete as f32,
                    y: vertical.discrete as f32,
                };

                if horizontal.discrete != 0 || vertical.discrete != 0 {
                    self.events
                        .push(IcedEvent::Mouse(iced::mouse::Event::WheelScrolled {
                            delta: scroll_delta,
                        }));
                }
            }
        }
    }

    pub fn take_events(&mut self) -> Vec<IcedEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn handle_keyboard_enter(&mut self) {
        trace!("[INPUT] Keyboard focus entered surface");
        // Iced doesn't have a direct WindowFocused event in the same way
        // This might be handled differently depending on the application needs
    }

    pub fn handle_keyboard_leave(&mut self) {
        trace!("[INPUT] Keyboard focus left surface");
        // Similar to above
    }

    pub fn handle_keyboard_event(&mut self, event: &KeyEvent, pressed: bool, is_repeat: bool) {
        trace!(
            "[INPUT] Keyboard event - keysym: {:?}, raw_code: {}, pressed: {}, repeat: {}, utf8: \
             {:?}",
            event.keysym.raw(),
            event.raw_code,
            pressed,
            is_repeat,
            event.utf8
        );

        // Check for clipboard operations BEFORE general key handling
        if pressed && !is_repeat && self.modifiers.contains(IcedModifiers::CTRL) {
            match event.keysym {
                Keysym::c => {
                    self.events
                        .push(IcedEvent::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: Key::Named(Named::Copy),
                            location: keysym_location(event.keysym),
                            modifiers: self.modifiers,
                            text: None,
                            modified_key: Key::Named(Named::Copy),
                            physical_key: Physical::Unidentified(NativeCode::Xkb(event.raw_code)),
                            repeat: false,
                        }));
                    return;
                }
                Keysym::x => {
                    self.events
                        .push(IcedEvent::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: Key::Named(Named::Cut),
                            location: keysym_location(event.keysym),
                            modifiers: self.modifiers,
                            text: None,
                            modified_key: Key::Named(Named::Cut),
                            physical_key: Physical::Unidentified(NativeCode::Xkb(event.raw_code)),
                            repeat: false,
                        }));
                    return;
                }
                Keysym::v => {
                    self.events
                        .push(IcedEvent::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: Key::Named(Named::Paste),
                            location: keysym_location(event.keysym),
                            modifiers: self.modifiers,
                            text: None,
                            modified_key: Key::Named(Named::Paste),
                            physical_key: Physical::Unidentified(NativeCode::Xkb(event.raw_code)),
                            repeat: false,
                        }));
                    return;
                }
                _ => (),
            }
        }

        let (key, location) = keysym_to_iced_key_and_loc(event.keysym);

        // Process keyboard event if we have a named key OR if we have text to input
        let text = if pressed || is_repeat {
            let mut text = event.utf8.clone();
            if is_repeat && text.is_none() {
                text = self.last_key_utf8.clone();
            }
            if let Some(ref text) = text {
                if !text.chars().any(|c| c.is_control()) {
                    trace!("[INPUT] Text input: '{}'", text);
                    Some(text.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ref t) = text {
            self.last_key_utf8 = Some(t.clone());
        }

        // Convert text from String to SmolStr if available
        let text_field = text.as_ref().map(|s| SmolStr::from(s.clone()));

        // Only emit events if we have a named key or text to input
        if !matches!(key, Key::Unidentified) || text_field.is_some() {
            trace!(
                "[INPUT] Mapped to Iced key: {:?}, repeat: {}, location: {:?}, has_text: {}",
                key,
                is_repeat,
                location,
                text_field.is_some()
            );

            let iced_event = if pressed {
                IcedEvent::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: key.clone(),
                    location,
                    modifiers: self.modifiers,
                    text: text_field,
                    modified_key: key,
                    physical_key: Physical::Unidentified(NativeCode::Xkb(event.raw_code)),
                    repeat: is_repeat,
                })
            } else {
                IcedEvent::Keyboard(iced::keyboard::Event::KeyReleased {
                    key,
                    location,
                    modifiers: self.modifiers,
                    physical_key: Physical::Unidentified(NativeCode::Xkb(event.raw_code)),
                    modified_key: Key::Unidentified,
                })
            };

            self.events.push(iced_event);
        }
    }

    pub fn update_modifiers(&mut self, wayland_mods: &WaylandModifiers) {
        trace!(
            "[INPUT] Modifiers updated - ctrl: {}, shift: {}, alt: {}",
            wayland_mods.ctrl, wayland_mods.shift, wayland_mods.alt
        );
        let mut mods = IcedModifiers::empty();
        if wayland_mods.shift {
            mods |= IcedModifiers::SHIFT;
        }
        if wayland_mods.ctrl {
            mods |= IcedModifiers::CTRL;
        }
        if wayland_mods.alt {
            mods |= IcedModifiers::ALT;
        }
        self.modifiers = mods;
    }

    pub fn get_modifiers(&self) -> IcedModifiers {
        self.modifiers
    }

    pub fn get_pointer_position(&self) -> (f64, f64) {
        self.pointer_pos
    }
}

fn keysym_to_iced_key(keysym: Keysym) -> Key {
    let named = match keysym {
        // TTY function keys
        Keysym::BackSpace => Named::Backspace,
        Keysym::Tab => Named::Tab,
        Keysym::Clear => Named::Clear,
        Keysym::Return => Named::Enter,
        Keysym::Pause => Named::Pause,
        Keysym::Scroll_Lock => Named::ScrollLock,
        Keysym::Sys_Req => Named::PrintScreen,
        Keysym::Escape => Named::Escape,
        Keysym::Delete => Named::Delete,

        // IME keys
        Keysym::Multi_key => Named::Compose,
        Keysym::Codeinput => Named::CodeInput,
        Keysym::SingleCandidate => Named::SingleCandidate,
        Keysym::MultipleCandidate => Named::AllCandidates,
        Keysym::PreviousCandidate => Named::PreviousCandidate,

        // Japanese key
        Keysym::Kanji => Named::KanjiMode,
        Keysym::Muhenkan => Named::NonConvert,
        Keysym::Henkan_Mode => Named::Convert,
        Keysym::Romaji => Named::Romaji,
        Keysym::Hiragana => Named::Hiragana,
        Keysym::Hiragana_Katakana => Named::HiraganaKatakana,
        Keysym::Zenkaku => Named::Zenkaku,
        Keysym::Hankaku => Named::Hankaku,
        Keysym::Zenkaku_Hankaku => Named::ZenkakuHankaku,
        Keysym::Kana_Lock => Named::KanaMode,
        Keysym::Kana_Shift => Named::KanaMode,
        Keysym::Eisu_Shift => Named::Alphanumeric,
        Keysym::Eisu_toggle => Named::Alphanumeric,

        // Cursor control & motion
        Keysym::Home => Named::Home,
        Keysym::Left => Named::ArrowLeft,
        Keysym::Up => Named::ArrowUp,
        Keysym::Right => Named::ArrowRight,
        Keysym::Down => Named::ArrowDown,
        Keysym::Page_Up => Named::PageUp,
        Keysym::Page_Down => Named::PageDown,
        Keysym::End => Named::End,

        // Misc. functions
        Keysym::Select => Named::Select,
        Keysym::Print => Named::PrintScreen,
        Keysym::Execute => Named::Execute,
        Keysym::Insert => Named::Insert,
        Keysym::Undo => Named::Undo,
        Keysym::Redo => Named::Redo,
        Keysym::Menu => Named::ContextMenu,
        Keysym::Find => Named::Find,
        Keysym::Cancel => Named::Cancel,
        Keysym::Help => Named::Help,
        Keysym::Break => Named::Pause,
        Keysym::Mode_switch => Named::ModeChange,
        Keysym::Num_Lock => Named::NumLock,

        // Keypad keys
        Keysym::KP_Tab => Named::Tab,
        Keysym::KP_Enter => Named::Enter,
        Keysym::KP_F1 => Named::F1,
        Keysym::KP_F2 => Named::F2,
        Keysym::KP_F3 => Named::F3,
        Keysym::KP_F4 => Named::F4,
        Keysym::KP_Home => Named::Home,
        Keysym::KP_Left => Named::ArrowLeft,
        Keysym::KP_Up => Named::ArrowUp,
        Keysym::KP_Right => Named::ArrowRight,
        Keysym::KP_Down => Named::ArrowDown,
        Keysym::KP_Page_Up => Named::PageUp,
        Keysym::KP_Page_Down => Named::PageDown,
        Keysym::KP_End => Named::End,
        Keysym::KP_Insert => Named::Insert,
        Keysym::KP_Delete => Named::Delete,

        // Function keys
        Keysym::F1 => Named::F1,
        Keysym::F2 => Named::F2,
        Keysym::F3 => Named::F3,
        Keysym::F4 => Named::F4,
        Keysym::F5 => Named::F5,
        Keysym::F6 => Named::F6,
        Keysym::F7 => Named::F7,
        Keysym::F8 => Named::F8,
        Keysym::F9 => Named::F9,
        Keysym::F10 => Named::F10,
        Keysym::F11 => Named::F11,
        Keysym::F12 => Named::F12,
        Keysym::F13 => Named::F13,
        Keysym::F14 => Named::F14,
        Keysym::F15 => Named::F15,
        Keysym::F16 => Named::F16,
        Keysym::F17 => Named::F17,
        Keysym::F18 => Named::F18,
        Keysym::F19 => Named::F19,
        Keysym::F20 => Named::F20,
        Keysym::F21 => Named::F21,
        Keysym::F22 => Named::F22,
        Keysym::F23 => Named::F23,
        Keysym::F24 => Named::F24,
        Keysym::F25 => Named::F25,
        Keysym::F26 => Named::F26,
        Keysym::F27 => Named::F27,
        Keysym::F28 => Named::F28,
        Keysym::F29 => Named::F29,
        Keysym::F30 => Named::F30,
        Keysym::F31 => Named::F31,
        Keysym::F32 => Named::F32,
        Keysym::F33 => Named::F33,
        Keysym::F34 => Named::F34,
        Keysym::F35 => Named::F35,

        // Modifiers
        Keysym::Shift_L => Named::Shift,
        Keysym::Shift_R => Named::Shift,
        Keysym::Control_L => Named::Control,
        Keysym::Control_R => Named::Control,
        Keysym::Caps_Lock => Named::CapsLock,
        Keysym::Alt_L => Named::Alt,
        Keysym::Alt_R => Named::Alt,
        Keysym::Super_L => Named::Super,
        Keysym::Super_R => Named::Super,
        Keysym::Hyper_L => Named::Hyper,
        Keysym::Hyper_R => Named::Hyper,

        // XKB function and modifier keys
        Keysym::ISO_Level3_Shift => Named::AltGraph,
        Keysym::ISO_Level3_Latch => Named::AltGraph,
        Keysym::ISO_Level3_Lock => Named::AltGraph,
        Keysym::ISO_Next_Group => Named::GroupNext,
        Keysym::ISO_Prev_Group => Named::GroupPrevious,
        Keysym::ISO_First_Group => Named::GroupFirst,
        Keysym::ISO_Last_Group => Named::GroupLast,
        Keysym::ISO_Left_Tab => Named::Tab,
        Keysym::ISO_Enter => Named::Enter,

        // 3270 terminal keys
        Keysym::_3270_EraseEOF => Named::EraseEof,
        // Keysym::_3270_Quit => Named::Quit, // Not available in current iced version
        Keysym::_3270_Attn => Named::Attn,
        Keysym::_3270_Play => Named::Play,
        Keysym::_3270_ExSelect => Named::ExSel,
        Keysym::_3270_CursorSelect => Named::CrSel,
        Keysym::_3270_PrintScreen => Named::PrintScreen,
        Keysym::_3270_Enter => Named::Enter,

        Keysym::space => Named::Space,

        // XFree86 - Backlight controls
        Keysym::XF86_MonBrightnessUp => Named::BrightnessUp,
        Keysym::XF86_MonBrightnessDown => Named::BrightnessDown,

        // XFree86 - "Internet"
        Keysym::XF86_Standby => Named::Standby,
        Keysym::XF86_AudioLowerVolume => Named::AudioVolumeDown,
        Keysym::XF86_AudioRaiseVolume => Named::AudioVolumeUp,
        Keysym::XF86_AudioPlay => Named::MediaPlay,
        Keysym::XF86_AudioStop => Named::MediaStop,
        Keysym::XF86_AudioPrev => Named::MediaTrackPrevious,
        Keysym::XF86_AudioNext => Named::MediaTrackNext,
        Keysym::XF86_HomePage => Named::BrowserHome,
        Keysym::XF86_Mail => Named::LaunchMail,
        Keysym::XF86_Search => Named::BrowserSearch,
        Keysym::XF86_AudioRecord => Named::MediaRecord,

        // XFree86 - PDA
        Keysym::XF86_Calculator => Named::LaunchApplication2,
        Keysym::XF86_Calendar => Named::LaunchCalendar,
        Keysym::XF86_PowerDown => Named::Power,

        // XFree86 - More "Internet"
        Keysym::XF86_Back => Named::BrowserBack,
        Keysym::XF86_Forward => Named::BrowserForward,
        Keysym::XF86_Refresh => Named::BrowserRefresh,
        Keysym::XF86_PowerOff => Named::Power,
        Keysym::XF86_WakeUp => Named::WakeUp,
        Keysym::XF86_Eject => Named::Eject,
        Keysym::XF86_ScreenSaver => Named::LaunchScreenSaver,
        Keysym::XF86_WWW => Named::LaunchWebBrowser,
        Keysym::XF86_Sleep => Named::Standby,
        Keysym::XF86_Favorites => Named::BrowserFavorites,
        Keysym::XF86_AudioPause => Named::MediaPause,
        Keysym::XF86_MyComputer => Named::LaunchApplication1,
        Keysym::XF86_AudioRewind => Named::MediaRewind,
        Keysym::XF86_Calculater => Named::LaunchApplication2, // libxkbcommon typo
        Keysym::XF86_Close => Named::Close,
        Keysym::XF86_Copy => Named::Copy,
        Keysym::XF86_Cut => Named::Cut,
        Keysym::XF86_Excel => Named::LaunchSpreadsheet,
        Keysym::XF86_LogOff => Named::LogOff,
        Keysym::XF86_MySites => Named::BrowserFavorites,
        Keysym::XF86_New => Named::New,
        Keysym::XF86_Open => Named::Open,
        Keysym::XF86_Paste => Named::Paste,
        Keysym::XF86_Phone => Named::LaunchPhone,
        Keysym::XF86_Reply => Named::MailReply,
        Keysym::XF86_Reload => Named::BrowserRefresh,
        Keysym::XF86_Save => Named::Save,
        Keysym::XF86_Send => Named::MailSend,
        Keysym::XF86_Spell => Named::SpellCheck,
        Keysym::XF86_SplitScreen => Named::SplitScreenToggle,
        Keysym::XF86_Video => Named::LaunchMediaPlayer,
        Keysym::XF86_Word => Named::LaunchWordProcessor,
        Keysym::XF86_ZoomIn => Named::ZoomIn,
        Keysym::XF86_ZoomOut => Named::ZoomOut,
        Keysym::XF86_WebCam => Named::LaunchWebCam,
        Keysym::XF86_MailForward => Named::MailForward,
        Keysym::XF86_Music => Named::LaunchMusicPlayer,
        Keysym::XF86_AudioForward => Named::MediaFastForward,
        Keysym::XF86_AudioRandomPlay => Named::RandomToggle,
        Keysym::XF86_Subtitle => Named::Subtitle,
        Keysym::XF86_AudioCycleTrack => Named::MediaAudioTrack,
        Keysym::XF86_Suspend => Named::Standby,
        Keysym::XF86_Hibernate => Named::Hibernate,
        Keysym::XF86_AudioMute => Named::AudioVolumeMute,
        Keysym::XF86_Next_VMode => Named::VideoModeNext,

        // Sun keyboard keys
        Keysym::SUN_Copy => Named::Copy,
        Keysym::SUN_Open => Named::Open,
        Keysym::SUN_Paste => Named::Paste,
        Keysym::SUN_Cut => Named::Cut,
        Keysym::SUN_AudioLowerVolume => Named::AudioVolumeDown,
        Keysym::SUN_AudioMute => Named::AudioVolumeMute,
        Keysym::SUN_AudioRaiseVolume => Named::AudioVolumeUp,
        Keysym::SUN_VideoLowerBrightness => Named::BrightnessDown,
        Keysym::SUN_VideoRaiseBrightness => Named::BrightnessUp,

        _ => return Key::Unidentified,
    };

    Key::Named(named)
}

fn keysym_location(keysym: Keysym) -> Location {
    match keysym {
        Keysym::Shift_L | Keysym::Control_L | Keysym::Alt_L | Keysym::Super_L | Keysym::Hyper_L => {
            Location::Left
        }
        Keysym::Shift_R | Keysym::Control_R | Keysym::Alt_R | Keysym::Super_R | Keysym::Hyper_R => {
            Location::Right
        }
        Keysym::KP_0
        | Keysym::KP_1
        | Keysym::KP_2
        | Keysym::KP_3
        | Keysym::KP_4
        | Keysym::KP_5
        | Keysym::KP_6
        | Keysym::KP_7
        | Keysym::KP_8
        | Keysym::KP_9
        | Keysym::KP_Space
        | Keysym::KP_Tab
        | Keysym::KP_Enter
        | Keysym::KP_F1
        | Keysym::KP_F2
        | Keysym::KP_F3
        | Keysym::KP_F4
        | Keysym::KP_Home
        | Keysym::KP_Left
        | Keysym::KP_Up
        | Keysym::KP_Right
        | Keysym::KP_Down
        | Keysym::KP_Page_Up
        | Keysym::KP_Page_Down
        | Keysym::KP_End
        | Keysym::KP_Begin
        | Keysym::KP_Insert
        | Keysym::KP_Delete
        | Keysym::KP_Equal
        | Keysym::KP_Multiply
        | Keysym::KP_Add
        | Keysym::KP_Separator
        | Keysym::KP_Subtract
        | Keysym::KP_Decimal
        | Keysym::KP_Divide => Location::Numpad,
        _ => Location::Standard,
    }
}

pub fn keysym_to_iced_key_and_loc(keysym: Keysym) -> (Key, Location) {
    let key = keysym_to_iced_key(keysym);
    let location = keysym_location(keysym);
    (key, location)
}

fn wayland_button_to_iced(button: u32) -> Option<IcedMouseButton> {
    // Linux button codes (from linux/input-event-codes.h)
    // BTN_LEFT = 0x110 = 272
    // BTN_RIGHT = 0x111 = 273
    // BTN_MIDDLE = 0x112 = 274
    match button {
        0x110 => Some(IcedMouseButton::Left),
        0x111 => Some(IcedMouseButton::Right),
        0x112 => Some(IcedMouseButton::Middle),
        _ => None,
    }
}
