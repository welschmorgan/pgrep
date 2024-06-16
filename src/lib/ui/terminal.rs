use std::{
  io::Stdout, panic::{set_hook, take_hook}, path::PathBuf, process::Command, time::Duration
};

use crate::{Error, Project, UI};

use crossterm::{
  event::{self, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::{log, Level};
use ratatui::{
  backend::CrosstermBackend,
  layout::{Constraint, Layout, Rect},
  style::{palette::tailwind, Modifier, Style},
  terminal::{Frame, Terminal as RataTerm},
  widgets::{Block, HighlightSpacing, List, ListState, Paragraph},
};

/// The `ncurses` interface, which allows having a user-friendly TUI in the terminal.
/// 
/// Activate with the `tui` feature **and** the `--tui` option.
pub struct Terminal<'a> {
  term: RataTerm<CrosstermBackend<Stdout>>,
  projects: Vec<Project>,
  projects_widget: List<'a>,
  projects_state: ListState,
  details_opened: bool,
  editor: Option<PathBuf>
}

impl<'a> Terminal<'a> {
  /// Create a `Terminal` instance.
  /// This will:
  ///   - Install panic hooks
  ///   - Setup cooked mode
  pub fn new(editor: Option<PathBuf>) -> crate::Result<Self> {
    Self::init_panic_hook();
    let term = Self::init_tui()?;
    Ok(Self {
      term,
      projects: vec![],
      projects_widget: List::new::<Vec<String>>(vec![]),
      projects_state: ListState::default(),
      details_opened: false,
      editor
    })
  }

  /// Setup cooked mode
  /// 
  /// https://www.gnu.org/software/mit-scheme/documentation/stable/mit-scheme-ref/Terminal-Mode.html
  fn init_tui() -> crate::Result<RataTerm<CrosstermBackend<Stdout>>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()
      .map_err(|e| Error::IO(format!("failed to enable raw mode"), Some(Box::new(e))))?;
    execute!(stdout, EnterAlternateScreen).map_err(|e| {
      Error::IO(
        format!("unable to enter alternate screen"),
        Some(Box::new(e)),
      )
    })?;
    RataTerm::new(CrosstermBackend::new(stdout))
      .map_err(|e| Error::IO(format!("failed to create terminal"), Some(Box::new(e))))
  }

  /// Install a panic hook to restore the terminal to raw mode before printing it.
  fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
      // intentionally ignore errors here since we're already in a panic
      let _ = Self::restore_tui();
      original_hook(panic_info);
    }));
  }

  /// Render a single terminal frame.
  /// 
  /// This will be called in a loop.
  pub fn render_frame(
    projects: &Vec<Project>,
    details_opened: bool,
    widget: &List,
    state: &mut ListState,
    frame: &mut Frame,
  ) -> crate::Result<()> {
    let constraints: &[Constraint] = match details_opened {
      true => &[Constraint::Percentage(20), Constraint::Percentage(80)],
      false => &[Constraint::Percentage(100)],
    };
    let frame_size = frame.size();
    let main_rect = Rect::new(
      frame_size.x,
      frame_size.y,
      frame_size.width,
      frame_size.height - 1,
    );
    let layout = Layout::horizontal(constraints).split(main_rect);
    frame.render_stateful_widget(widget, layout[0], state);
    if constraints.len() == 2 {
      let proj = &projects[state.selected().unwrap_or_default()];
      let details_text = format!(
        "Languages: {}\nName: {}\nPath: {}",
        proj
          .kinds()
          .iter()
          .map(|k| k.name())
          .collect::<Vec<_>>()
          .join(","),
        proj.name().unwrap_or_default(),
        proj.path().display()
      );
      let details = Paragraph::new(details_text).block(Block::bordered().title("Details"));
      frame.render_widget(details, layout[1]);
    }
    let menu_rect = Rect::new(frame_size.x, frame_size.height - 1, frame_size.width, 1);
    let menu_layout = Layout::horizontal(&[
      Constraint::Percentage(25),
      Constraint::Percentage(25),
      Constraint::Percentage(25),
    ])
    .split(menu_rect);
    frame.render_widget(Paragraph::new("[Q]uit"), menu_layout[0]);
    frame.render_widget(Paragraph::new("Toggle details (Return)"), menu_layout[1]);
    frame.render_widget(Paragraph::new("[O]pen project"), menu_layout[2]);
    Ok(())
  }

  /// Retore the terminal to it's raw mode
  /// 
  /// https://www.gnu.org/software/mit-scheme/documentation/stable/mit-scheme-ref/Terminal-Mode.html
  fn restore_tui(/* term: &mut RataTerm<CrosstermBackend<Stdout>> */) -> crate::Result<()> {
    let mut stdout = std::io::stdout();
    disable_raw_mode()
      .map_err(|e| Error::IO(format!("failed to disable raw mode"), Some(Box::new(e))))?;
    execute!(stdout, LeaveAlternateScreen).map_err(|e| {
      Error::IO(
        format!("failed to switch to main screen"),
        Some(Box::new(e)),
      )
    })?;
    Ok(())
  }
}

/// Restore the terminal to it's raw mode when dropped
impl<'a> Drop for Terminal<'a> {
  fn drop(&mut self) {
    let _ = Self::restore_tui();
    let _ = self
      .term
      .show_cursor()
      .map_err(|e| Error::IO(format!("unable to show cursor"), Some(Box::new(e))));
  }
}

impl<'a> UI for Terminal<'a> {
  fn write_matches(
    &mut self,
    matches: &Vec<crate::Project>,
    _fmt: &crate::BoxedProjectMatchesFormatter,
  ) -> crate::Result<()> {
    self.projects.append(&mut matches.clone());
    self.projects_widget = List::new(self.projects.iter().map(|proj| {
      let kinds = proj
        .kinds()
        .iter()
        .map(|k| k.name())
        .collect::<Vec<_>>()
        .join(",");
      let name = proj.name().unwrap_or_default();
      let path = format!("{}", proj.path().display());
      format!("[{}] {} - {}", kinds, name, path)
    }))
    .block(Block::bordered().title(format!("Projects ({})", self.projects.len())))
    .highlight_style(
      Style::default()
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::REVERSED)
        .fg(tailwind::BLUE.c300),
    )
    .highlight_symbol(">")
    .highlight_spacing(HighlightSpacing::Always);
    self.projects_state = ListState::default().with_selected(match self.projects.is_empty() {
      true => None,
      false => Some(0),
    });
    Ok(())
  }

  fn write_log(&mut self, _text: &str, _lvl: log::Level) -> crate::Result<()> {
    unimplemented!("log messages display")
  }

  fn render_loop(&mut self) -> crate::Result<()> {
    loop {
      self.term.draw(|frame| {
        Self::render_frame(
          &self.projects,
          self.details_opened,
          &self.projects_widget,
          &mut self.projects_state,
          frame,
        )
        .unwrap()
      })?;
      if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
          if KeyCode::Char('q') == key.code {
            break;
          } else if KeyCode::Up == key.code {
            let cur_sel = self.projects_state.selected().unwrap_or_default();
            if !self.projects.is_empty() && cur_sel > 0 {
              self.projects_state.select(Some(cur_sel - 1));
            }
          } else if KeyCode::Down == key.code {
            let cur_sel = self.projects_state.selected().unwrap_or_default();
            if !self.projects.is_empty() && cur_sel < self.projects.len() - 1 {
              self.projects_state.select(Some(cur_sel + 1));
            }
          } else if KeyCode::Enter == key.code {
            self.details_opened = !self.details_opened;
          } else if KeyCode::Char('o') == key.code {
            let editor = self.editor.clone()
                .or_else(|| std::env::var("EDITOR").ok().map(|v| PathBuf::from(v)))
                .or_else(|| std::env::var("VISUAL").ok().map(|v| PathBuf::from(v)));
            let editor = match editor {
              Some(editor) => editor,
              None => {
                panic!("EDITOR or VISUAL environment variable missing, --editor missing please define it first.")
              }
            };
            let proj = &self.projects[self.projects_state.selected().unwrap_or_default()];
            let cmd = Command::new(editor)
              .arg(format!("{}", proj.path().display()))
              .spawn()?;
            let output = cmd.wait_with_output()?;
            let stdout = String::from_utf8(output.stdout)?;
            let stderr = String::from_utf8(output.stderr)?;
            if !output.status.success() {
                self.write_log(&vec![stdout, stderr].join("\n"), Level::Error)?;
            }
          }
        }
      }
    }
    Ok(())
  }
}
