use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;

use flume::Receiver;
use flume::Sender;
use hecs::TypeIdMap;
use hecs::{Entity, World};
use mana_tui_elemental::layout::Props;
use mana_tui_utils::resource::Resources;
use mana_tui_utils::systems::SystemsExt;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::EventStream;
use ratatui::style::Style;
use smallvec::SmallVec;
use tokio_stream::StreamExt;

use crate::focus::Clicked;
use crate::focus::FocusState;
use crate::schedule::PostRenderSchedule;
use crate::schedule::PreRenderSchedule;

pub mod focus;
pub mod schedule;

pub fn handle_event(mut world: &mut World, event: Event) {
    match event {
        Event::FocusGained => {}
        Event::FocusLost => {}
        Event::Key(key_event) => {
            focus::keybind_clicked_system(world, key_event);
            focus::handle_pressed(world);
        }
        Event::Mouse(mouse_event) => {
            focus::clear_old_hovers(world);
            _ = focus::handle_mouse_event(world, mouse_event);
            focus::on_click_system(world);
            world.run_systems::<PostRenderSchedule>();
            focus::click_post_update_system(world);
            {}
        }
        Event::Paste(_) => {}
        Event::Resize(_, _) => {}
    }
}

pub fn setup_interactions(mut world: &mut World, root: Entity) {
    world.run_systems::<PreRenderSchedule>();
    focus::generate_ui_stack(world, root);
}

enum UiEvent {
    ClickedStyleFinished(TypeId),
}

#[derive(derive_more::Deref, derive_more::DerefMut, Clone)]
struct EventQueue((Sender<UiEvent>, Receiver<UiEvent>));

pub fn init(world: &mut World) {
    world.insert_resource(EventQueue(flume::unbounded()));
}

/// # Panics
///
/// Panics if the user hasn't called [`init`].
pub async fn read(world: &mut World) -> Option<Event> {
    let queue = {
        let queue = world
            .get_resource::<&EventQueue>()
            .expect("event queue not found: use mana-tui-beheaded::init first");

        queue.deref().clone()
    };

    let mut event_stream = EventStream::new();

    loop {
        tokio::select! {
            inner_event = queue.1.recv_async() => {
                let Ok(inner_event) = inner_event else {
                    continue;
                };
                handle_ui_event(world, inner_event);
                return None;
            }
            crossterm_event = event_stream.next() => {
                let Some(Ok(event)) = crossterm_event else {
                    continue;
                };
                handle_event(world, event.clone());
                return Some(event);
            }
        };
    }
}

pub(crate) fn handle_ui_event(world: &mut World, event: UiEvent) {
    match event {
        UiEvent::ClickedStyleFinished(typeid) => {
            let mut focus = world
                .get_resource::<&mut Store<FocusState>>()
                .expect("focus state resource must be initialized");
            if let Some(focus_state) = focus.get_mut(&typeid) {
                focus_state.current_style = focus_state.normal_style;
            }
        }
    }
}

#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct Store<T> {
    map: TypeIdMap<T>,
}

impl<T> Store<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: TypeIdMap::default(),
        }
    }
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Marker(TypeId);
