use console::truncate_str;
use laurier::{key_code, key_code_char};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{List, ListItem, Paragraph},
    Frame, Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{util::digits, Action, Target, TargetKind};

#[derive(Default)]
pub struct Tui {
    targets: Vec<Target>,
    filtered: Vec<FilteredTarget>,
    cursor: usize,
    input: Input,
    action: Action,

    list_height: usize,
    list_offset: usize,
}

struct FilteredTarget {
    index: usize,
    match_indices: Vec<usize>,
}

pub enum Ret {
    Quit,
    Selected(Target, Action),
    NotSelected,
}

impl Tui {
    pub fn new(targets: Vec<Target>, term_size: Rect) -> Tui {
        let mut tui = Tui {
            targets,
            list_height: Tui::calc_list_height(term_size.height),
            ..Default::default()
        };
        tui.update_filter();
        tui
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<Ret> {
        loop {
            terminal.draw(|f| self.render(f))?;

            match event::read()? {
                Event::Key(key) => match key {
                    key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => {
                        return Ok(Ret::Quit);
                    }
                    key_code!(KeyCode::Down) | key_code_char!('n', Ctrl) => {
                        self.select_next();
                    }
                    key_code!(KeyCode::Up) | key_code_char!('p', Ctrl) => {
                        self.select_prev();
                    }
                    key_code!(KeyCode::Tab) => {
                        self.toggle_action();
                    }
                    key_code!(KeyCode::Enter) => {
                        let ret = match self.get_current_target() {
                            Some(target) => Ret::Selected(target, self.action),
                            None => Ret::NotSelected,
                        };
                        return Ok(ret);
                    }
                    _ => {
                        self.input.handle_event(&Event::Key(key));
                        self.update_filter();
                    }
                },
                Event::Resize(_, h) => {
                    self.list_height = Tui::calc_list_height(h);
                }
                _ => {}
            }
        }
    }

    fn calc_list_height(h: u16) -> usize {
        (h - 1) as usize
    }

    fn select_next(&mut self) {
        if self.cursor < self.filtered.len() - 1 {
            if self.cursor - self.list_offset == self.list_height - 1 {
                self.list_offset += 1;
            }
            self.cursor += 1;
        }
    }

    fn select_prev(&mut self) {
        if self.cursor > 0 {
            if self.cursor - self.list_offset == 0 {
                self.list_offset -= 1;
            }
            self.cursor -= 1;
        }
    }

    fn toggle_action(&mut self) {
        self.action = match self.action {
            Action::Run => Action::Build,
            Action::Build => Action::Run,
        };
    }

    fn get_current_target(&self) -> Option<Target> {
        self.filtered
            .get(self.cursor)
            .and_then(|t| self.targets.get(t.index))
            .cloned()
    }

    fn update_filter(&mut self) {
        let s = self.input.value();
        self.filtered = self
            .targets
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                t.name.find(s).map(|pos| {
                    let match_indices = (pos..pos + s.len()).collect();
                    FilteredTarget {
                        index: i,
                        match_indices,
                    }
                })
            })
            .collect();
        self.cursor = 0;
        self.list_offset = 0;
    }

    fn render(&self, f: &mut Frame) {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(f.area());
        self.render_input(f, chunks[0]);
        self.render_list(f, chunks[1]);
    }

    fn render_input(&self, f: &mut Frame, area: Rect) {
        let targets_num_digits = digits(self.targets.len());
        let max_w = area.width as usize;
        let label_w = 7;
        let num_w = targets_num_digits * 2 + 5;
        let input_w = max_w - (label_w + num_w + 3);

        let (label, label_color) = match self.action {
            Action::Run => ("  run  ", Color::Green),
            Action::Build => (" build ", Color::Blue),
        };
        let input = format!("{:input_w$}", self.input.value());
        let nums = if self.filtered.is_empty() {
            "".to_string()
        } else {
            format!(
                "({:targets_num_digits$} / {:targets_num_digits$})",
                self.cursor + 1,
                self.filtered.len()
            )
        };
        let spans = vec![
            label.bg(label_color).fg(Color::Black),
            " ".into(),
            input.into(),
            " ".into(),
            nums.fg(Color::DarkGray),
            " ".into(),
        ];
        let line = Paragraph::new(Line::from(spans));
        f.render_widget(line, area);

        let x = area.x + 8 + (self.input.visual_cursor() as u16);
        let y = area.y;
        f.set_cursor_position((x, y));
    }

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let max_w = area.width as usize;
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .enumerate()
            .skip(self.list_offset)
            .take(self.list_height)
            .flat_map(|(i, ft)| {
                let selected = i == self.cursor;
                self.targets
                    .get(ft.index)
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
