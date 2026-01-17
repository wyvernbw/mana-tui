use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Flex;
use ratatui::layout::Layout;
use ratatui::layout::Margin;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use tachyonfx::Duration;
use tachyonfx::Effect;
use tachyonfx::EffectRenderer;
use tui_term::widget::PseudoTerminal;

use crate::AppBridge;
use crate::AppStage;
use crate::RendererBuildState;
use crate::RendererState;

pub struct AppFx {
    pub(crate) title_hsl_shift: Option<Effect>,
    pub(crate) text_glitch_progress: f32,
}

impl AppBridge {
    pub(crate) fn get_pty_area(&self, area: Rect) -> Rect {
        let width = area.height * self.aspect.0 / self.aspect.1;
        Layout::horizontal([Constraint::Max(width)])
            .flex(Flex::Center)
            .areas::<1>(area)[0]
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    pub(crate) fn draw(&self, frame: &mut Frame, state: &mut RendererState, dt: Duration) {
        let running_app = state.running_app.as_ref().map_or("", |v| v);
        let title_text = format!("running {running_app}");
        let title_len = title_text.len();
        let title_text = format!(" 📺 {} ", title_text);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dim())
            .padding(Padding::uniform(1))
            .title_top(title_text);
        if let Some(fx) = &mut state.app_fx.title_hsl_shift {
            let [title_area] =
                Layout::new(Direction::Vertical, [Constraint::Length(1)]).areas(frame.area());
            let [_, title_area] = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Length(32 + 2),
                    Constraint::Length(title_len as u16),
                ],
            )
            .areas(title_area);
            frame.render_effect(fx, title_area, dt);
        }
        let area = self.get_pty_area(frame.area()).outer(Margin {
            horizontal: 1,
            vertical: 1,
        });
        frame.render_widget(&block, area);
        let screen_area = block.inner(area);

        match state.stage {
            AppStage::StaringIpc => {
                let loading = Paragraph::new(format!("Loading {running_app}..."))
                    .centered()
                    .style(Style::new().dim());
                let screen_area = screen_area.centered_vertically(Constraint::Length(1));
                frame.render_widget(loading, screen_area);
            }
            AppStage::Building(RendererBuildState::Building {
                build_max_progress,
                build_progress,
            }) => {
                let loading = Paragraph::new(format!("Building {running_app}..."));
                frame.render_widget(loading, screen_area);
            }
            AppStage::Building(RendererBuildState::Idle) => {
                let loading = Paragraph::new(format!("Built {running_app}."));
                frame.render_widget(loading, screen_area);
            }
            AppStage::Running => {
                if let Some(screen) = &state.screen {
                    let term = PseudoTerminal::new(&**screen);
                    // let term = Paragraph::new("I am terminal").centered();
                    frame.render_widget(term, screen_area);
                }
            }
        }
    }
}
