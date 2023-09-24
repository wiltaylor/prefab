use std::io::Stdout;
use std::time::Duration;
use anyhow::anyhow;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::{Color, Span, Style, Stylize};
use ratatui::Terminal;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, ListItem, Paragraph};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use prefab::template::TemplateOption;
use crate::tui::option_ui::{EditorStatus, OptionUi};

pub struct NumberUI {
    option: TemplateOption,
    input: Input,
    status: EditorStatus,
    is_float: bool,
}

impl NumberUI {
    pub fn new(option: TemplateOption) -> NumberUI {
        match &option {
            TemplateOption::Integer { prompt, value, mandatory } => {
                let val = if value.is_some() {
                    format!("{}", value.unwrap())
                } else {
                    "".to_string()
                };

                NumberUI {
                    option: TemplateOption::Integer { prompt: prompt.clone(), value: *value, mandatory: false },
                    input: Input::from(val),
                    status: EditorStatus::Continue,
                    is_float: false,
                }
            }
            TemplateOption::Float { prompt, value, mandatory } => {
                let val = if value.is_some() {
                    format!("{}", value.unwrap())
                } else {
                    "".to_string()
                };

                NumberUI {
                    option: TemplateOption::Float { prompt: prompt.clone(), value: *value, mandatory: false },
                    input: Input::from(val),
                    status: EditorStatus::Continue,
                    is_float: true,
                }
            }
            _ => panic!("Expected a number option type!")
        }
    }
}

impl OptionUi for NumberUI {
    fn render_list_item(&self) -> anyhow::Result<ListItem> {
        match &self.option {
            TemplateOption::Integer { prompt, value, mandatory } =>
                Ok(ListItem::new(if let Some(value) = value {
                Line::from(vec![
                    Span::raw(prompt).green(),
                    Span::raw(" => ").yellow(),
                    Span::raw(format!("{}", value)).white(),
                ])
            } else {
                Line::from(vec![
                    Span::raw(prompt).green(),
                    Span::raw(" => ").yellow(),
                    Span::raw("Empty").gray(),
                ])
            })),
            TemplateOption::Float { prompt, value, mandatory } =>
                Ok(ListItem::new(if let Some(value) = value {
                Line::from(vec![
                    Span::raw(prompt).green(),
                    Span::raw(" => ").yellow(),
                    Span::raw(format!("{}", value)).white(),
                ])
            } else {
                Line::from(vec![
                    Span::raw(prompt).green(),
                    Span::raw(" => ").yellow(),
                    Span::raw("Empty").gray(),
                ])
            })),
            _ => Err(anyhow!("Option is not a number field!"))
        }
    }

    fn render_edit(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {

        let val = self.input.value().to_string();
        let prompt = match &self.option {
            TemplateOption::Integer { prompt, value:_, mandatory } => prompt,
            TemplateOption::Float { prompt, value:_, mandatory } => prompt,
            _ => return Err(anyhow!("Option is not a integer or float!"))
        };

        let text = vec![
            Line::from(vec![Span::raw(prompt).green(), Span::raw(":").yellow()]),
            Line::from(vec![Span::raw(val.clone())])
        ];

        terminal.draw(|frame| {
            let paragraph = Paragraph::new(text)
                .block(Block::default().title("[Edit]-(Enter: Confirm, Esq - Cancel) ").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));

            frame.set_cursor((val.len() as u16) + 1, 2);
            frame.render_widget(paragraph, frame.size())

        })?;
        Ok(())
    }

    fn update_input(&mut self) -> anyhow::Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Esc == key.code {
                    self.status = EditorStatus::Cancel;
                    return Ok(())
                } else if KeyCode::Enter == key.code {
                    let v = self.input.value().clone().to_string();
                    let o = self.option.clone().set_value(v);
                    self.option = o.clone();
                    self.status = EditorStatus::Finished { option: o };
                    return Ok(());
                } else {
                    let orig = self.input.value().clone().to_string();

                    self.input.handle_event(&Event::Key(key));
                    let v = self.input.value().clone().to_string();

                    if v.is_empty() || v == "-".to_string() ||(self.is_float && v.parse::<f64>().is_ok()) || v.parse::<i64>().is_ok(){
                        //Keep value
                    }else {
                        self.input = Input::new(orig);
                    }
                }
            }
        }

        Ok(())

    }

    fn get_status(&self) -> anyhow::Result<EditorStatus> { Ok(self.status.clone())     }

    fn start_edit(&mut self) {
        self.status = EditorStatus::Continue;

        if let Some(v) = self.option.get_value() {
            self.input = Input::new(v);
        }else{
            self.input = Input::default();
        }
    }

    fn get_option(&self) -> TemplateOption { self.option.clone() }
}