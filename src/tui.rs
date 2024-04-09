use console::truncate_str;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
    Frame, Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{key_code, key_code_char, Target, TargetKind};

pub struct Tui {
    targets: Vec<Target>,
    filtered: Vec<usize>,
    cursor: usize,
    input: Input,
}

impl Tui {
    pub fn new(targets: Vec<Target>) -> Tui {
        let filtered = (0..targets.len()).collect();
        Tui {
            targets,
            filtered,
            cursor: 0,
            input: Input::default(),
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                match key {
                    key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => {
                        return Ok(());
                    }
                    key_code_char!('n', Ctrl) => {
                        if self.cursor < self.filtered.len() - 1 {
                            self.cursor += 1;
                        }
                    }
                    key_code_char!('p', Ctrl) => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    key_code!(KeyCode::Enter) => {
                        // todo
                    }
                    _ => {
                        self.input.handle_event(&Event::Key(key));
                        self.update_filter();
                    }
                }
            }
        }
    }

    fn update_filter(&mut self) {
        let s = self.input.value();
        self.filtered = self
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name.contains(s))
            .map(|(i, _)| i)
            .collect();
        self.cursor = 0;
    }

    fn render(&self, f: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(f.size());
        self.render_input(f, chunks[0]);
        self.render_list(f, chunks[1]);
    }

    fn render_input(&self, f: &mut Frame, area: Rect) {
        let spans = vec![
            Span::styled(" run ", Style::default().bg(Color::Green).fg(Color::Black)),
            " ".into(),
            self.input.value().into(),
        ];
        let line = Paragraph::new(Line::from(spans));
        f.render_widget(line, area);

        let x = area.x + 6 + (self.input.visual_cursor() as u16);
        let y = area.y;
        f.set_cursor(x, y);
    }

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let max_w = area.width as usize;
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .enumerate()
            .flat_map(|(i, fi)| {
                let selected = i == self.cursor;
                self.targets
                    .get(*fi)
                    .map(|t| self.build_list_item(t, selected, max_w))
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, area);
    }

    fn build_list_item(&self, target: &Target, selected: bool, max_w: usize) -> ListItem {
        let kind_w: usize = 7;
        let name_w: usize = 25;
        let path_w: usize = 30;
        let features_w: usize = max_w - (kind_w + name_w + path_w + 5);

        let kind = match target.kind {
            TargetKind::Bin => "bin",
            TargetKind::Example => "example",
        };
        let name = truncate_str(&target.name, name_w, "..");
        let path = truncate_str(&target.path, path_w, "..");
        let features = if target.required_features.is_empty() {
            "".to_string()
        } else {
            let s = format!("--features {:?}", target.required_features);
            truncate_str(&s, features_w, "..").into()
        };

        let spans = vec![
            " ".into(),
            format!("{:kind_w$}", kind).fg(Color::Blue),
            " ".into(),
            format!("{:name_w$}", name).fg(Color::White),
            " ".into(),
            format!("{:path_w$}", path).fg(Color::DarkGray),
            " ".into(),
            format!("{:features_w$}", features).fg(Color::DarkGray),
            " ".into(),
        ];
        let line = Text::from(Line::from(spans));
        let style = if selected {
            Style::default().bg(Color::Yellow)
        } else {
            Style::default()
        };
        ListItem::new(line).style(style)
    }
}
