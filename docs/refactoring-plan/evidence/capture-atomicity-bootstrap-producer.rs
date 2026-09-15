//! Small real terminal producer for capture atomicity qualification, not TC.
use std::io::{self, IsTerminal, Read, Write};

struct App {
    value: u64,
    generation: u64,
    tick: u64,
}

impl App {
    fn update(&mut self) {
        self.value += 1;
        self.generation += 1;
        self.tick += 7;
    }

    fn draw(&self, output: &mut impl Write) -> io::Result<()> {
        let phase = if self.value % 2 == 0 { 'A' } else { 'B' };
        let color = if phase == 'A' { 31 } else { 34 };
        write!(output, "\x1b[0m\x1b[2J\x1b[H\x1b[{color}m{phase}:{}", self.value)?;
        if phase == 'A' {
            write!(output, "\x1b[2;3H\x1b[?25h")?;
        } else {
            write!(output, "\x1b[3;8H\x1b[?25l")?;
        }
        Ok(())
    }

    fn checkpoint(&self, output: &mut impl Write, nonce: &str) -> io::Result<()> {
        self.draw(output)?;
        // Same stream, after all render bytes. No further update can occur
        // until the observer acknowledges this barrier with another input.
        let phase = if self.value % 2 == 0 { 'A' } else { 'B' };
        write!(output, "\x1b]777;{nonce};{{\"generation\":{},\"action_index\":{},\"tick\":{},\"value\":{},\"phase\":\"{phase}\"}}\x07",
            self.generation, self.generation, self.tick, self.value)?;
        output.flush()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("real PTY required".into());
    }
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().ok_or("missing seed")?.parse()?;
    let nonce = args.next().ok_or("missing barrier nonce")?;
    if seed > 1_000_000 || nonce.len() != 64 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) || args.next().is_some() {
        return Err("invalid frozen producer arguments".into());
    }
    let mut app = App { value: seed, generation: 0, tick: seed * 7 };
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    app.checkpoint(&mut output, &nonce)?;
    loop {
        let mut key = [0];
        input.read_exact(&mut key)?;
        match key[0] {
            b'+' => { app.update(); app.checkpoint(&mut output, &nonce)?; }
            b'q' => return Ok(()),
            _ => return Err("unexpected frozen input".into()),
        }
    }
}
