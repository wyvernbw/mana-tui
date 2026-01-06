//! # ratatui-elemental
//!
//! ratatui layout library

#![forbid(missing_docs)]

pub(crate) mod layout;

/// prelude module. contains public api for `ratatui-elemental`.
///
/// # Usage
///
/// ```
/// use ratatui_elemental::prelude::*;
/// ```
pub mod prelude {
    use ratatui::{
        layout::{Direction, Rect},
        widgets::{Block, BorderType, Borders, Padding, Paragraph},
    };

    use crate::layout::{ElWidget, ElementCtx, ElementIdx, LayoutParams, Size};

    /// create element builder.
    ///
    /// an element is a unit in the layout system. elements have children
    /// and form a tree that whose layout can be rendered by the context.
    ///
    /// # params
    /// - `widget`: the widget to be rendered
    ///
    /// # methods
    /// - `create`: construct the element
    /// - `width`: sizing along the x axis
    /// - `height`: sizing along the y axis,
    /// - `direction`: layout direction for children
    /// - `padding`: padding around around children
    /// - `gap`: gap between children on the main axis
    #[bon::builder]
    #[builder(finish_fn = create)]
    pub fn element(
        #[builder(start_fn)] widget: impl ElWidget + 'static,
        #[builder(finish_fn)] ctx: &mut ElementCtx,
        #[builder(overwritable)] layout_params: Option<LayoutParams>,
        #[builder(default, overwritable)] width: Size,
        #[builder(default, overwritable)] height: Size,
        #[builder(default, overwritable)] direction: Direction,
        #[builder(overwritable)] padding: Option<Padding>,
        #[builder(default, overwritable)] padding_left: u16,
        #[builder(default, overwritable)] padding_right: u16,
        #[builder(default, overwritable)] padding_top: u16,
        #[builder(default, overwritable)] padding_bottom: u16,
        #[builder(default, overwritable)] gap: u16,
        children: Option<&[ElementIdx]>,
    ) -> ElementIdx {
        let layout_params = layout_params.unwrap_or(LayoutParams {
            width,
            height,
            direction,
            padding: padding.unwrap_or(Padding {
                left: padding_left,
                right: padding_right,
                top: padding_top,
                bottom: padding_bottom,
            }),
            gap,
        });
        ElementCtx::element(widget)
            .maybe_children(children)
            .layout_params(layout_params)
            .create(ctx)
    }

    /// return type of [`block`]
    ///
    /// see [`element`] for more options.
    pub type BlockElBuilder =
        ElementBuilder<'static, 'static, Block<'static>, element_builder::Empty>;

    /// function for creating [`Block`] structs with sensible defaults around borders.
    ///
    /// this function will create an [`ElementBuilder`] that has its padding set to 1
    /// on all sides where the block has a border. this ensures that children elements
    /// do not draw over the block's borders.
    ///
    /// see [`element`] for more options.
    pub fn block(block: Block<'static>) -> BlockElBuilder {
        // FIXME: when ratatui exposes `Block::borders`
        let test_area = Rect {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };
        let inner_area = block.inner(test_area);
        let left = inner_area.left() - test_area.left();
        let top = inner_area.top() - test_area.top();
        let right = test_area.height - inner_area.height;
        let bottom = test_area.height - inner_area.height;

        element(block).padding(Padding {
            left,
            right,
            top,
            bottom,
        })
    }

    /// like [`block`], but sets the borders to [`BorderType::Rounded`] and [`Borders::ALL`].
    ///
    /// see [`element`] for more options.
    pub fn block_rounded(bl: Block<'static>) -> BlockElBuilder {
        block(bl.border_type(BorderType::Rounded).borders(Borders::all()))
    }

    /// marker trait implemented for any `ElementBuilder`.
    ///
    /// it is used to extend builders with other methods. you can also use it to target
    /// implementations of extension traits for your own needs.
    ///
    /// ```
    /// use ratatui_elemental::prelude::*;
    ///
    /// trait MyElementExt {
    ///     fn foo(&self);
    /// }
    ///
    /// impl<T: ElementalBuilder> MyElementExt for T {
    ///     # fn foo(&self) {}
    ///     /* ... */
    /// }
    /// ```
    pub trait ElementalBuilder {}

    impl<'f1, 'f2, W: ElWidget, S: element_builder::State> ElementalBuilder
        for ElementBuilder<'f1, 'f2, W, S>
    {
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::{Direction, Rect},
        style::{Color, Stylize},
        widgets::{Block, BorderType, Padding, Paragraph},
    };

    use crate::{
        layout::{ElementCtx, LayoutParams, Size},
        prelude::{block, block_rounded, element},
    };

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
        let root = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(Color::Red),
        )
        .layout_params(LayoutParams {
            width: Size::Fixed(24),
            height: Size::Fixed(8),
            ..Default::default()
        })
        .create(&mut ctx);
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_fixed_size\n{}", buffer_to_string(&buf));
    }

    #[test]
    fn test_fixed_size_with_children() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let root = block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("parent")
                .fg(Color::Red),
        )
        .children(&[
            block_rounded(Block::bordered().title_top("child #0".to_string()))
                .width(Size::Fixed(10))
                .height(Size::Fixed(3))
                .create(&mut ctx),
            block_rounded(Block::bordered().title_top("child #1".to_string()))
                .width(Size::Fixed(14))
                .height(Size::Fixed(3))
                .create(&mut ctx),
        ])
        .width(Size::Fixed(24))
        .height(Size::Fixed(8))
        .create(&mut ctx);

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
            ElementCtx::element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 20),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create(ctx)
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ElementCtx::element(
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
        .create(&mut ctx);
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
            ElementCtx::element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create(ctx)
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ElementCtx::element(
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
        .create(&mut ctx);
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_fit\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_horizontal() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        let mut ctx = ElementCtx::default();
        let child = |mut ctx: &mut ElementCtx, idx| {
            let ctx1 = &mut ctx;
            ElementCtx::element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create(ctx1)
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ElementCtx::element(
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
        .create(&mut ctx);
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
            ElementCtx::element(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(format!("child #{idx}")),
            )
            .layout_params(LayoutParams {
                width: Size::Fixed(10 + idx as u16 * 2),
                height: Size::Fixed(3),
                ..Default::default()
            })
            .create(ctx)
        };
        let children = &[child(&mut ctx, 0), child(&mut ctx, 1)];
        let root = ElementCtx::element(
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
        .create(&mut ctx);
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_gap\n{}", buffer_to_string(&buf));
    }
    #[test]
    fn test_grow() {
        _ = tracing_subscriber::fmt::try_init();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 24));
        let mut ctx = ElementCtx::default();
        let child0 = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("sidebar".to_string()),
        )
        .children(&[ElementCtx::element(
            Paragraph::new("this sidebar is so amazing it can have long text that wraps around")
                .wrap(ratatui::widgets::Wrap { trim: false }),
        )
        .layout_params(LayoutParams {
            width: Size::Grow,
            height: Size::Grow,
            ..Default::default()
        })
        .create(&mut ctx)])
        .layout_params(LayoutParams {
            width: Size::Fixed(10),
            padding: Padding::uniform(1),
            height: Size::Grow,
            ..Default::default()
        })
        .create(&mut ctx);
        let child2 = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("child #2".to_string()),
        )
        .layout_params(LayoutParams {
            width: Size::Grow,
            height: Size::Grow,
            ..Default::default()
        })
        .create(&mut ctx);
        let child3 = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("child #3".to_string()),
        )
        .layout_params(LayoutParams {
            width: Size::Grow,
            height: Size::Grow,
            ..Default::default()
        })
        .create(&mut ctx);
        let child1 = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("child #1".to_string()),
        )
        .layout_params(LayoutParams {
            width: Size::Grow,
            padding: Padding::uniform(1),
            gap: 1,
            height: Size::Grow,
            direction: Direction::Vertical,
        })
        .children(&[child2, child3])
        .create(&mut ctx);
        let root = ElementCtx::element(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top("parent")
                .fg(Color::Red),
        )
        .children(&[child0, child1])
        .layout_params(LayoutParams {
            width: Size::Fixed(36),
            height: Size::Fixed(18),
            direction: Direction::Horizontal,
            padding: Padding::uniform(1),
            ..Default::default()
        })
        .create(&mut ctx);
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
            ElementCtx::element(
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
            .create(ctx)
        };
        let subchildren = &[child(&mut ctx, 0, 6), child(&mut ctx, 1, 6)];
        let children = &[
            child(&mut ctx, 2, 14).children(&mut ctx, subchildren),
            child(&mut ctx, 3, 14),
        ];
        let root = ElementCtx::element(
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
        .create(&mut ctx);
        ctx.calculate_layout(root);
        ctx.render(root, buf.area, &mut buf);
        tracing::info!("\ntest_horizontal\n{}", buffer_to_string(&buf));
    }
}
