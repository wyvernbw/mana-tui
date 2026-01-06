use std::{any::Any, sync::Arc};

use derive_more as d;
use generational_arena::{Arena, Index};
use glam::{U16Vec2, u16vec2};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Margin, Rect},
    widgets::{Padding, StatefulWidget, Widget},
};
use smallbox::SmallBox;

type ElementArena = Arena<TuiElement>;
type WidgetBox = SmallBox<dyn ElWidget, usize>;
type WidgetArena = Arena<WidgetBox>;

trait ElWidget {
    fn render_element(&self, area: Rect, buf: &mut Buffer);
}

impl<W> ElWidget for W
where
    W: Widget + Clone,
{
    fn render_element(&self, area: Rect, buf: &mut Buffer) {
        self.clone().render(area, buf);
    }
}

#[derive(Default)]
struct ElementCtx {
    elements: ElementArena,
    widgets: WidgetArena,
}

impl std::ops::Index<ElementIdx> for ElementCtx {
    type Output = TuiElement;

    fn index(&self, index: ElementIdx) -> &Self::Output {
        &self.elements[*index]
    }
}

impl std::ops::IndexMut<ElementIdx> for ElementCtx {
    fn index_mut(&mut self, index: ElementIdx) -> &mut Self::Output {
        &mut self.elements[*index]
    }
}

#[bon::bon]
impl ElementCtx {
    #[builder(finish_fn = create)]
    pub fn element<W>(
        &mut self,
        #[builder(start_fn)] widget: W,
        #[builder(default)] layout_params: LayoutParams,
        children: Option<&[ElementIdx]>,
    ) -> ElementIdx
    where
        W: ElWidget + 'static,
    {
        let children = match children {
            Some(children) => children.to_vec(),
            None => Vec::default(),
        };
        let children = Arc::new(children);
        let widget_idx = self.widgets.insert(SmallBox::new(widget) as WidgetBox);
        let widget_idx = WidgetIdx(widget_idx);
        let element = TuiElement {
            widget: widget_idx,
            layout_params,
            size: U16Vec2::default(),
            position: U16Vec2::default(),
            children,
        };
        let element_idx = self.elements.insert(element);

        ElementIdx(element_idx)
    }
    fn calculate_fit_sizes(&mut self, element: ElementIdx) {
        let el = &mut self[element];
        match el.layout_params.width {
            Size::Fixed(size) => el.size.x = size,
            Size::Fit => {}
            Size::Grow => {}
        }
        match el.layout_params.height {
            Size::Fixed(size) => el.size.y = size,
            Size::Fit => {}
            Size::Grow => {}
        }
        let children = self[element].children.clone();
        let padding = self[element].layout_params.padding;
        let max_size = self[element].size.saturating_sub(u16vec2(
            padding.right + padding.left,
            padding.bottom + padding.top,
        ));
        let direction = self[element].layout_params.direction;
        let mut axis_sizes = AxisSizes::default();
        for child in children.iter().copied() {
            self.calculate_fit_sizes(child);
            if self[element].layout_params.width.should_clamp() {
                self[child].size.x = self[child].size.x.clamp(0, max_size.x);
            }
            if self[element].layout_params.width.should_clamp() {
                self[child].size.y = self[child].size.y.clamp(0, max_size.x);
            }
            axis_sizes = axis_sizes.increase(self[child].size, direction);
        }
        axis_sizes = axis_sizes.pad(padding, direction);
        axis_sizes.main_axis +=
            children.len().saturating_sub(1) as u16 * self[element].layout_params.gap;
        let axis_sizes = axis_sizes.to_u16vec2(direction);
        match self[element].layout_params.width {
            Size::Fixed(_) => {}
            Size::Fit | Size::Grow => {
                self[element].size.x = axis_sizes.x;
            }
        }
        match self[element].layout_params.height {
            Size::Fixed(_) => {}
            Size::Fit | Size::Grow => {
                self[element].size.y = axis_sizes.y;
            }
        }
        // for child in children.iter().copied() {
        //     self.calculate_fit_sizes(child);
        // }
    }
    fn calculate_grow_sizes(&mut self, element: ElementIdx) {
        let children = self[element].children.clone();
        let padding = self[element].layout_params.padding;
        let max_size = self[element].size.saturating_sub(u16vec2(
            padding.right + padding.left,
            padding.bottom + padding.top,
        ));
        let direction = self[element].layout_params.direction;
        let used_space = children
            .iter()
            .copied()
            .map(|child| self[child].size)
            .sum::<U16Vec2>();
        let remaining_size = self[element]
            .size
            .saturating_sub(used_space)
            .clamp(U16Vec2::ZERO, max_size);
        tracing::info!(?used_space, ?remaining_size);
        let mut remaining_size = axify(remaining_size, direction);

        // cross axis
        for child in children.iter().copied() {
            if self[child].layout_params.cross_size(direction).is_grow() {
                let mut size = AxisSizes::from_u16vec2(self[child].size, direction);
                size.cross_axis = remaining_size.cross_axis;
                self[child].size = size.to_u16vec2(direction);
            }
        }

        // main axis
        while remaining_size.main_axis > 0 {
            let mut smallest: [Option<ElementIdx>; 2] = [None, None];
            let mut first = None;
            let mut all_equal = true;
            let mut grow_count = 0;
            for child in children.iter().copied() {
                let size = self[child].size;
                let size = AxisSizes::from_u16vec2(size, direction);
                let is_grow = self[child].layout_params.main_size(direction).is_grow();
                if is_grow {
                    if first.is_some() && Some(size) != first {
                        all_equal = false;
                    }
                    grow_count += 1;
                }
                first = Some(size);
                match (&smallest, is_grow) {
                    (_, false) => {}
                    (&[None, None], true) => {
                        smallest[0] = Some(child);
                    }
                    (&[Some(a), None], true) => {
                        let asize = axify(self[a].size, direction);
                        if asize.main_axis < size.main_axis {
                            smallest[1] = Some(child);
                        } else if asize.main_axis != size.main_axis {
                            smallest[1] = smallest[0];
                            smallest[0] = Some(child);
                        }
                    }
                    (&[Some(a), Some(b)], true) => {
                        let asize = axify(self[a].size, direction);
                        let bsize = axify(self[b].size, direction);
                        if asize.main_axis < size.main_axis {
                            smallest[1] = smallest[0];
                            smallest[0] = Some(child);
                        } else if size.main_axis < bsize.main_axis {
                            smallest[1] = Some(child);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            if all_equal && grow_count > 0 {
                let remaining_size = remaining_size.main_axis / grow_count;
                for child in children.iter().copied() {
                    let mut size = axify(self[child].size, direction);
                    size.main_axis = remaining_size;
                    self[child].size = size.to_u16vec2(direction);
                }
                break;
            }
            match smallest {
                [Some(a), Some(b)] => {
                    let mut asize = axify(self[a].size, direction);
                    let bsize = axify(self[b].size, direction);
                    assert!(asize.main_axis != bsize.main_axis);
                    remaining_size = remaining_size.min(remaining_size - (bsize - asize));
                    asize.main_axis = remaining_size.main_axis;
                    self[a].size = asize.to_u16vec2(direction);
                }
                [Some(a), None] => {
                    let mut asize = axify(self[a].size, direction);
                    asize.main_axis = remaining_size.main_axis;
                    self[a].size = asize.to_u16vec2(direction);
                    break;
                }
                [None, None] => break,
                [None, Some(_)] => unreachable!(),
            }
        }

        for child in children.iter().copied() {
            self.calculate_grow_sizes(child);
        }
    }
    fn calculate_positions(&mut self, root: ElementIdx) {
        let dir = self[root].layout_params.direction;
        let children = self[root].children.clone();
        let padding = self[root].layout_params.padding;
        let gap = self[root].layout_params.gap;
        let mut axis_start = 0;
        for child in children.iter().copied() {
            self[child].position = self[root].position;
            match dir {
                Direction::Horizontal => self[child].position.x += axis_start,
                Direction::Vertical => self[child].position.y += axis_start,
            }
            self[child].position += u16vec2(padding.left, padding.top);
            axis_start = increase_axis(axis_start, dir, self[child].size);
            axis_start += gap;
            self.calculate_positions(child);
        }
    }
    pub(crate) fn calculate_layout(&mut self, element: ElementIdx) {
        self.calculate_fit_sizes(element);
        self.calculate_grow_sizes(element);
        self.calculate_positions(element);
    }
    pub(crate) fn render(&self, root: ElementIdx, area: Rect, buf: &mut Buffer) {
        let el = &self[root];
        let area = el.split_area(area);
        self.widgets[*el.widget].render_element(area, buf);
        for child in el.children.iter().copied() {
            tracing::info!(?child);
            self.render(child, area, buf);
        }
    }
}

fn increase_axis(init: u16, dir: Direction, size: U16Vec2) -> u16 {
    match dir {
        Direction::Horizontal => init + size.x,
        Direction::Vertical => init + size.y,
    }
}

fn get_axis_rect(position: U16Vec2, dir: Direction, rect: Rect) -> Rect {
    match dir {
        Direction::Horizontal => Rect {
            x: rect.x + position.x,
            width: rect.width.saturating_sub(position.x),
            ..rect
        },
        Direction::Vertical => Rect {
            y: rect.y + position.y,
            height: rect.height.saturating_sub(position.y),
            ..rect
        },
    }
}

#[derive(
    Debug, Clone, Copy, Default, d::Sub, d::SubAssign, d::Add, d::AddAssign, d::Sum, PartialEq, Eq,
)]
struct AxisSizes {
    main_axis: u16,
    cross_axis: u16,
}

const fn axify(vec: U16Vec2, dir: Direction) -> AxisSizes {
    AxisSizes::from_u16vec2(vec, dir)
}

impl AxisSizes {
    #[inline(always)]
    fn min(self, other: AxisSizes) -> AxisSizes {
        AxisSizes {
            main_axis: self.main_axis.min(other.main_axis),
            cross_axis: self.cross_axis.min(other.cross_axis),
        }
    }
    #[inline(always)]
    const fn from_u16vec2(value: U16Vec2, dir: Direction) -> Self {
        match dir {
            Direction::Horizontal => Self {
                main_axis: value.x,
                cross_axis: value.y,
            },
            Direction::Vertical => Self {
                main_axis: value.y,
                cross_axis: value.x,
            },
        }
    }
    #[inline(always)]
    const fn pad(self, padding: Padding, dir: Direction) -> AxisSizes {
        match dir {
            Direction::Horizontal => AxisSizes {
                main_axis: self.main_axis + padding.left + padding.right,
                cross_axis: self.cross_axis + padding.top + padding.bottom,
            },
            Direction::Vertical => AxisSizes {
                main_axis: self.main_axis + padding.top + padding.bottom,
                cross_axis: self.cross_axis + padding.left + padding.right,
            },
        }
    }
    #[inline(always)]
    fn increase(self, by: U16Vec2, dir: Direction) -> AxisSizes {
        match dir {
            Direction::Horizontal => AxisSizes {
                main_axis: self.main_axis + by.x,
                cross_axis: self.cross_axis.max(by.y),
            },
            Direction::Vertical => AxisSizes {
                main_axis: self.main_axis + by.y,
                cross_axis: self.cross_axis.max(by.x),
            },
        }
    }
    #[inline(always)]
    const fn to_u16vec2(self, dir: Direction) -> U16Vec2 {
        match dir {
            Direction::Horizontal => u16vec2(self.main_axis, self.cross_axis),
            Direction::Vertical => u16vec2(self.cross_axis, self.main_axis),
        }
    }
}

#[derive(d::Deref, d::From, Clone, Copy, Debug)]
struct WidgetIdx(Index);
#[derive(d::Deref, d::From, Clone, Copy, Debug)]
struct ElementIdx(Index);

impl ElementIdx {
    fn children(self, ctx: &mut ElementCtx, children: &[ElementIdx]) -> Self {
        ctx[self].children = Arc::new(children.to_vec());
        self
    }
}

struct TuiElement {
    widget: WidgetIdx,
    layout_params: LayoutParams,
    position: U16Vec2,
    size: U16Vec2,
    // FIXME: double pointer indirection
    children: Arc<Vec<ElementIdx>>,
}

#[derive(Default)]
struct LayoutParams {
    width: Size,
    height: Size,
    direction: Direction,
    padding: Padding,
    gap: u16,
}

impl LayoutParams {
    fn main_size(&self, dir: Direction) -> Size {
        match dir {
            Direction::Horizontal => self.width,
            Direction::Vertical => self.height,
        }
    }
    fn cross_size(&self, dir: Direction) -> Size {
        match dir {
            Direction::Horizontal => self.height,
            Direction::Vertical => self.width,
        }
    }
}

#[derive(Default, Clone, Copy)]
enum Size {
    Fixed(u16),
    #[default]
    Fit,
    Grow,
}

impl Size {
    fn should_clamp(&self) -> bool {
        match self {
            Size::Fixed(_) => true,
            Size::Fit => false,
            Size::Grow => true,
        }
    }
    fn is_grow(&self) -> bool {
        matches!(self, Size::Grow)
    }
}

impl TuiElement {
    fn split_area(&self, area: Rect) -> Rect {
        area.intersection(Rect {
            // DONE: implement position
            x: self.position.x,
            y: self.position.y,
            width: self.size.x,
            height: self.size.y,
        })
    }
}

#[cfg(test)]
mod tests {
    use glam::u16vec2;
    use ratatui::{
        buffer::Buffer,
        layout::{Direction, Rect},
        style::{Color, Stylize},
        widgets::{Block, BorderType, Padding},
    };

    use crate::{ElementCtx, LayoutParams, Size};

    fn buffer_to_string(buf: &Buffer) -> String {
        buf.content()
            .chunks(buf.area.width as usize)
            .flat_map(|line| line.iter().map(|cell| cell.symbol()).chain(["\n"]))
            .collect()
    }

    #[test]
    fn test_fixed_size() {
        let _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .fg(Color::Red),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(24),
                height: Size::Fixed(8),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_fixed_size\n{}", buffer_to_string(&buf));
    }

    #[test]
    fn test_fixed_size_with_children() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create()
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fixed(24),
                height: Size::Fixed(8),
                direction: Direction::Vertical,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!(
            "\ntest_fixed_size_with_children\n{}",
            buffer_to_string(&buf)
        );
    }

    #[test]
    fn test_fixed_size_with_children_clamp_children() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 20),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create()
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fixed(24),
                height: Size::Fixed(8),
                direction: Direction::Vertical,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!(
            "\ntest_fixed_size_with_children_clamp_children\n{}",
            buffer_to_string(&buf)
        );
    }

    #[test]
    fn test_fit() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create()
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fit,
                height: Size::Fit,
                direction: Direction::Vertical,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_fit\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_horizontal() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create()
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fit,
                height: Size::Fit,
                direction: Direction::Horizontal,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_horizontal\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_gap() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create()
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fit,
                height: Size::Fit,
                direction: Direction::Horizontal,
                padding: Padding::uniform(1),
                gap: 2,
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_gap\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_grow() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 24));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx| {};
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let child0 = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("child #0".to_string()),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10),
                height: Size::Grow,
                ..Default::default()
            })
            .create();
        let child2 = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("child #2".to_string()),
            )
            .layout_params(LayoutParams {
                width: Size::Grow,
                height: Size::Grow,
                ..Default::default()
            })
            .create();
        let child3 = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("child #3".to_string()),
            )
            .layout_params(LayoutParams {
                width: Size::Grow,
                height: Size::Grow,
                ..Default::default()
            })
            .create();
        let child1 = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("child #1".to_string()),
            )
            .layout_params(LayoutParams {
                width: Size::Grow,
                padding: Padding::uniform(1),
                height: Size::Grow,
                direction: Direction::Vertical,
                ..Default::default()
            })
            .children(&[child2, child3])
            .create();
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(&[child0, child1])
            .layout_params(LayoutParams {
                width: Size::Fixed(36),
                height: Size::Fixed(12),
                direction: Direction::Horizontal,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_grow\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_multiple_children() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 20));
        let mut ctx = ElementCtx::default();
        let child = |ctx: &mut ElementCtx, idx, height| {
            ctx.element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                padding: Padding::uniform(1),
                width: Size::Fixed(16),
                height: Size::Fixed(height),
                ..Default::default()
            })
            .create()
        };
        let subchildren = &[child(&mut ctx, 0, 6), child(&mut ctx, 1, 6)];
        let children = &[
            child(&mut ctx, 2, 14).children(&mut ctx, subchildren),
            child(&mut ctx, 3, 14),
        ];
        let root = ctx
            .element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top("parent")
                    .fg(Color::Red),
            )
            .children(children)
            .layout_params(LayoutParams {
                width: Size::Fit,
                height: Size::Fit,
                direction: Direction::Horizontal,
                padding: Padding::uniform(1),
                ..Default::default()
            })
            .create();
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_horizontal\n{}", buffer_to_string(&buf));
    }
}
