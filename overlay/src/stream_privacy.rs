//! Private rendering for the OBS browser source.
use std::{borrow::Cow, cell::Cell, collections::{HashMap, HashSet}, sync::{Mutex, OnceLock}};

thread_local! {
    static PRIVATE: Cell<bool> = const { Cell::new(false) };
}

pub fn active() -> bool {
    PRIVATE.with(Cell::get)
}

pub fn render_private<T>(render: impl FnOnce() -> T) -> T {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            PRIVATE.with(|flag| flag.set(self.0));
        }
    }
    let _reset = Reset(PRIVATE.with(|flag| flag.replace(true)));
    render()
}

use crate::class_icons::Role;
use baras_types::StreamAliases;

#[derive(Default)]
struct Aliases {
    templates: StreamAliases,
    mode: NameMode,
    revealed: HashSet<i64>,
    names: HashMap<String, (usize, usize)>,
    counts: [usize; 4],
}
static ALIASES: OnceLock<Mutex<Aliases>> = OnceLock::new();

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum NameMode {
    Private,
    #[default]
    Selected,
    Public,
}

static REVISION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn revision() -> u64 {
    REVISION.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn mode() -> NameMode {
    ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).mode
}

pub fn set_mode(mode: NameMode) {
    ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).mode = mode;
    REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn is_revealed(id: i64) -> bool {
    ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).revealed.contains(&id)
}

pub fn toggle_player(id: i64) {
    let mut state = ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner());
    if !state.revealed.remove(&id) {
        state.revealed.insert(id);
    }
    REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn game_role_icon(role: Option<Role>) -> Option<&'static crate::class_icons::ClassIcon> {
    if !active() || !ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).templates.use_game_role_icons {
        return None;
    }
    crate::class_icons::get_role_icon(match role? {
        Role::Damage => "icon_dps",
        Role::Tank => "icon_tank",
        Role::Healer => "icon_heal",
    })
}

pub fn german() -> bool {
    ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).templates.german
}

pub fn set_templates(templates: StreamAliases) {
    ALIASES.get_or_init(Mutex::default).lock()
        .unwrap_or_else(|error| error.into_inner()).templates = templates;
}

pub fn alias(name: &str) -> Cow<'_, str> {
    player_label(name, None, false, None)
}

pub fn player_label(name: &str, role: Option<Role>, is_local: bool, id: Option<i64>) -> Cow<'_, str> {
    if !active() || name.trim().is_empty() {
        return Cow::Borrowed(name);
    }
    let mut state = ALIASES.get_or_init(Mutex::default)
        .lock().unwrap_or_else(|error| error.into_inner());
    let reveal = match state.mode {
        NameMode::Private => is_local,
        NameMode::Selected => is_local || id.is_some_and(|id| state.revealed.contains(&id)),
        NameMode::Public => true,
    };
    if reveal {
        let template = match role {
            Some(Role::Damage) => &state.templates.damage,
            Some(Role::Tank) => &state.templates.tank,
            Some(Role::Healer) => &state.templates.healer,
            None => return Cow::Borrowed(name),
        };
        // Reuse a leading symbol, including emoji variation selectors and joiners.
        let symbol = template.trim_start()
            .split_once("{n}").map_or(template.trim_start(), |(prefix, _)| prefix);
        let end = symbol.find(|ch: char| ch.is_alphanumeric() || ch.is_whitespace())
            .unwrap_or(symbol.len());
        return if end == 0 { Cow::Borrowed(name) } else {
            Cow::Owned(format!("{} {name}", &symbol[..end]))
        };
    }
    let key = name.trim().to_lowercase();
    let category = match role {
        Some(Role::Damage) => 0,
        Some(Role::Tank) => 1,
        Some(Role::Healer) => 2,
        None => 3,
    };
    let (category, number) = match state.names.get(&key).copied() {
        Some(value) if value.0 != 3 || category == 3 => value,
        _ => {
            state.counts[category] += 1;
            let value = (category, state.counts[category]);
            state.names.insert(key, value);
            value
        }
    };
    let template = match category {
        0 => &state.templates.damage,
        1 => &state.templates.tank,
        2 => &state.templates.healer,
        _ if state.templates.german => "Spieler {n}",
        _ => "Player {n}",
    };
    let number = number.to_string();
    Cow::Owned(if template.trim().is_empty() {
        format!("{} {number}", if state.templates.german { "Spieler" } else { "Player" })
    } else if template.contains("{n}") {
        template.replace("{n}", &number)
    } else {
        format!("{} {number}", template.trim())
    })
}
