use std::{
    io,
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use crossterm::event::{self, Event::Key, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

use crate::util::transpose_iter;

use self::components::Plot;

mod components;
mod tui;

#[derive(Default)]
pub enum State {
    #[default]
    Train,
    Error(&'static str),
    Quit,
}

pub struct Update {
    pub episode: u16,
    pub data: Vec<Vec<(f64, f64)>>,
}

pub struct App {
    state: State,
    episode: u16,
    total_episodes: u16,
    plot_names: Vec<String>,
    plots: Vec<Plot>,
    selected_plot: usize,
}

impl App {
    pub fn new(plots: &[&str], episodes: u16) -> Self {
        Self {
            state: Default::default(),
            episode: 0,
            total_episodes: episodes,
            plot_names: plots.iter().map(|p| String::from(*p)).collect(),
            plots: plots
                .into_iter()
                .map(|p| Plot::new(*p).with_x_bounds([0.0, episodes as f64]))
                .collect(),
            selected_plot: 0,
        }
    }

    pub fn run(&mut self, rx: Receiver<Update>) -> io::Result<()> {
        let mut terminal = tui::init()?;

        loop {
            match self.state {
                State::Train => {
                    match rx.try_recv() {
                        Ok(Update { episode, data }) => {
                            self.episode = episode;
                            for (i, metric) in transpose_iter(data).enumerate() {
                                self.plots[i].update(metric);
                            }
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            self.state = State::Error("Channel disconnected.");
                        }
                    };

                    terminal.draw(|frame| frame.render_widget(&*self, frame.size()))?;

                    if event::poll(Duration::from_millis(16))? {
                        if let Key(key) = event::read()? {
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }
                            match key.code {
                                KeyCode::Left => {
                                    let len = self.plots.len();
                                    self.selected_plot = (self.selected_plot + len - 1) % len;
                                }
                                KeyCode::Right => {
                                    self.selected_plot =
                                        (self.selected_plot + 1) % self.plots.len();
                                }
                                KeyCode::Char('q') => {
                                    self.state = State::Quit;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                State::Error(_) => todo!(),
                State::Quit => break,
            }
        }

        tui::restore()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)])
            .split(area);

        // let horz = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        //     .split(vert[0]);

        // let bottom = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints([Constraint::Fill(1), Constraint::Length(3)])
        //     .split(vert[1]);

        Gauge::default()
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Progress"),
            )
            .ratio(self.episode as f64 / self.total_episodes as f64)
            .render(vert[1], buf);

        Tabs::new(self.plot_names.iter().map(String::as_str))
            .block(Block::default().padding(Padding::uniform(2)))
            .white()
            .highlight_style(Style::default().yellow())
            .select(self.selected_plot)
            .render(vert[0], buf);

        // Block::bordered()
        //     .border_type(BorderType::Rounded)
        //     .title("Info")
        //     .render(horz[0], buf);

        self.plots[self.selected_plot].render(vert[0], buf);
    }
}
