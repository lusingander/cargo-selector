use console::truncate_str;
use laurier::{highlight::highlight_matched_text, key_code, key_code_char};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
    Frame, Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{config::ColorTheme, matcher::Matcher, util::digits, Action, Target, TargetKind};

const ELLIPSIS: &str = "..";

#[derive(Default)]
pub struct Tui {
    targets: Vec<Target>,
    filtered: Vec<FilteredTarget>,
    cursor: usize,
    input: Input,
    action: Action,

    list_height: usize,
    list_offset: usize,

    matcher: Matcher,
    theme: ColorTheme,
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
    pub fn new(targets: Vec<Target>, term_size: Rect, matcher: Matcher, theme: ColorTheme) -> Tui {
        let mut tui = Tui {
            targets,
            list_height: Tui::calc_list_height(term_size.height),
            matcher,
            theme,
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
                self.matcher
                    .match_indices(&t.name, s)
                    .map(|indices| FilteredTarget {
                        index: i,
                        match_indices: indices,
                    })
            })
            .collect();
        self.cursor = 0;
        self.list_offset = 0;
    }

    fn render(&self, f: &mut Frame) {
        let block = Block::default().bg(self.theme.bg);
        f.render_widget(block, f.area());

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

        let (label, label_bg, label_fg) = match self.action {
            Action::Run => (
                "  run  ",
                self.theme.action_run_bg,
                self.theme.action_run_fg,
            ),
            Action::Build => (
                " build ",
                self.theme.action_build_bg,
                self.theme.action_build_fg,
            ),
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
            label.bg(label_bg).fg(label_fg),
            " ".into(),
            input.fg(self.theme.input_fg),
            " ".into(),
            nums.fg(self.theme.numbers_fg),
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
                    .map(|t| self.build_list_item(t, selected, max_w, &ft.match_indices))
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, area);
    }

    fn build_list_item(
        &self,
        target: &Target,
        selected: bool,
        max_w: usize,
        matched_indices: &[usize],
    ) -> ListItem {
        let kind_w: usize = 7;
        let name_w: usize = 25;
        let path_w: usize = 30;
        let features_w: usize = max_w - (kind_w + name_w + path_w + 5);

        let kind = match target.kind {
            TargetKind::Bin => "bin",
            TargetKind::Example => "example",
        };
        let name = truncate_str(&target.name, name_w, ELLIPSIS);
        let path = truncate_str(&target.path, path_w, ELLIPSIS);
        let features = if target.required_features.is_empty() {
            "".to_string()
        } else {
            let s = format!("--features {:?}", target.required_features);
            truncate_str(&s, features_w, ELLIPSIS).into()
        };

        let mut name_mt = highlight_matched_text(name.to_string())
            .matched_indices(matched_indices.to_vec())
            .not_matched_style(Style::default().fg(self.theme.name_fg))
            .matched_style(Style::default().fg(self.theme.name_match_fg));
        if name.ends_with(ELLIPSIS) {
            name_mt = name_mt.ellipsis(ELLIPSIS);
        }
        let mut name_spans = name_mt.into_spans();
        if name.len() < name_w {
            name_spans.push(" ".repeat(name_w - name.len()).into());
        }

        let mut spans = Vec::new();
        spans.push(" ".into());
        spans.push(format!("{:kind_w$}", kind).fg(self.theme.kind_fg));
        spans.push(" ".into());
        spans.extend(name_spans);
        spans.push(" ".into());
        spans.push(format!("{:path_w$}", path).fg(self.theme.path_fg));
        spans.push(" ".into());
        spans.push(format!("{:features_w$}", features).fg(self.theme.features_fg));
        spans.push(" ".into());

        let line = Text::from(Line::from(spans));
        let style = if selected {
            Style::default().bg(self.theme.selected_bg)
        } else {
            Style::default()
        };
        ListItem::new(line).style(style)
    }
}
