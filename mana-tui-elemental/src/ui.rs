//! helpers to create ui nodes
//!
//! # Usage
//!
//! ```
//! # use ratatui::widgets::Block;
//! # use mana_tui_elemental::ui::*;
//! # use mana_tui_elemental::prelude::*;
//!
//! let mut ctx = ElementCtx::new();
//! let root = ui(Block::new())
//!     .with((Width(Size::Grow), Height(Size::Fixed(40))))
//!     .children((
//!         ui(Block::new()),
//!         ui(Block::new())
//!     ));
//! ctx.spawn_ui(root);
//!
//! ```

use std::{collections::VecDeque, sync::Arc};

use glam::U16Vec2;
use hecs::{DynamicBundle, EntityBuilder};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    widgets::Padding,
};

use crate::layout::{
    Children, ElWidget, Element, ElementCtx, Gap, Height, Justify, MainJustify, Props, Size,
    TuiElMarker, Width,
};

#[bon::builder]
#[builder(finish_fn = done)]
pub fn ui<W: ElWidget>(
    #[builder(start_fn)] widget: W,
    #[builder(field = EntityBuilder::new())] mut builder: EntityBuilder,
) -> EntityBuilder {
    fn render_system<E: ElWidget>(
        ctx: &ElementCtx,
        entity: hecs::Entity,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if let Ok(widget) = ctx.world.get::<&E>(entity) {
            widget.render_element(area, buf);
        }
    }
    builder.add(widget);
    builder.add_bundle((
        TuiElMarker,
        Props {
            size: U16Vec2::default(),
            position: U16Vec2::default(),
            render: render_system::<W>,
        },
    ));
    if !builder.has::<Width>() {
        builder.add(Width(Size::Fit));
    }
    if !builder.has::<Height>() {
        builder.add(Height(Size::Fit));
    }
    if !builder.has::<Direction>() {
        builder.add(Direction::Vertical);
    }
    if !builder.has::<MainJustify>() {
        builder.add(MainJustify(Justify::Start));
    }
    if !builder.has::<Gap>() {
        builder.add(Gap::default());
    }
    if !builder.has::<Padding>() {
        builder.add(Padding::default());
    }
    if !builder.has::<ChildrenBuilders>() {
        builder.add(Children::None);
    }
    builder
}

impl<W, S> UiBuilder<W, S>
where
    S: ui_builder::IsComplete,
    W: ElWidget,
{
    pub fn children(
        mut self,
        children: impl IntoUiBuilderList,
    ) -> UiBuilder<W, impl ui_builder::State> {
        let children = children.into_list().into_iter().collect::<Box<[_]>>();
        self.builder.add(ChildrenBuilders(children));
        self
    }

    pub fn child(
        mut self,
        child: impl Into<EntityBuilder>,
    ) -> UiBuilder<W, impl ui_builder::State> {
        self.builder.add(ChildrenBuilders(Box::new([child.into()])));
        self
    }

    pub fn with(mut self, bundle: impl DynamicBundle) -> UiBuilder<W, impl ui_builder::State> {
        self.builder.add_bundle(bundle);
        self
    }
}

impl<W, S> From<UiBuilder<W, S>> for EntityBuilder
where
    S: ui_builder::IsComplete,
    W: ElWidget,
{
    fn from(val: UiBuilder<W, S>) -> Self {
        val.done()
    }
}

pub trait IntoUiBuilderList {
    fn into_list(self) -> impl IntoIterator<Item = EntityBuilder>;
}

impl<U> IntoUiBuilderList for Vec<U>
where
    U: Into<EntityBuilder>,
{
    fn into_list(self) -> impl IntoIterator<Item = EntityBuilder> {
        self.into_iter().map(|value| value.into())
    }
}

impl<const N: usize, U> IntoUiBuilderList for [U; N]
where
    U: Into<EntityBuilder>,
{
    fn into_list(self) -> impl IntoIterator<Item = EntityBuilder> {
        self.into_iter().map(|value| value.into())
    }
}

macro_rules! impl_into_ui_builder_list_for_tuples {
    ($($idx:tt $name:ident),+) => {
        impl<$($name),+> IntoUiBuilderList for ($($name,)+)
        where
            $($name: Into<EntityBuilder>,)+
        {
            fn into_list(self) -> impl IntoIterator<Item = EntityBuilder> {
                [$(self.$idx.into()),+]
            }
        }
    };
}

// Generate implementations for tuples of size 1 through 12
impl_into_ui_builder_list_for_tuples!(0 U0);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6, 7 U7);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6, 7 U7, 8 U8);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6, 7 U7, 8 U8, 9 U9);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6, 7 U7, 8 U8, 9 U9, 10 U10);
impl_into_ui_builder_list_for_tuples!(0 U0, 1 U1, 2 U2, 3 U3, 4 U4, 5 U5, 6 U6, 7 U7, 8 U8, 9 U9, 10 U10, 11 U11);

pub(crate) struct ChildrenBuilders(pub(crate) Box<[EntityBuilder]>);

fn process_ui_system(world: &mut ElementCtx) {
    let mut to_process: VecDeque<Element> = world
        .query_mut::<&ChildrenBuilders>()
        .into_iter()
        .map(|(e, _)| e)
        .collect();

    while let Some(node) = to_process.pop_front() {
        if let Ok(builders) = world.remove_one::<ChildrenBuilders>(node) {
            let mut builders = builders.0;
            world.reserve_entities(builders.len() as u32);
            let children = builders
                .iter_mut()
                .map(|builder| {
                    let builder = builder.build();
                    let has_children = builder.has::<ChildrenBuilders>();
                    let entity = world.spawn(builder);
                    if has_children {
                        to_process.push_back(entity);
                    }
                    entity
                })
                .collect();
            world
                .insert_one(node, Children::Some(Arc::new(children)))
                .unwrap();
        }
    }
}

impl ElementCtx {
    pub fn spawn_ui(&mut self, ui: impl Into<EntityBuilder>) -> Element {
        let mut ui = ui.into();
        let ui = ui.build();
        let root = self.spawn(ui);
        process_ui_system(self);
        root
    }
}
