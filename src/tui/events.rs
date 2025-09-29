use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum InputEvent {
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    Quit,
}

pub async fn handle_events(tx: mpsc::UnboundedSender<InputEvent>) -> anyhow::Result<()> {
    loop {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    if let KeyCode::Char('q') = key_event.code {
                        if key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            let _ = tx.send(InputEvent::Quit);
                            break;
                        }
                    }
                    let _ = tx.send(InputEvent::Key(key_event));
                }
                Event::Resize(cols, rows) => {
                    let _ = tx.send(InputEvent::Resize(cols, rows));
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
    Ok(())
}
