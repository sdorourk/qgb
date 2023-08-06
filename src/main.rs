use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// ROM program to run
    program: PathBuf,
    /// Boot ROM
    #[arg(short, long)]
    boot_rom: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    init_logger();

    let rom = fs::read(&cli.program).unwrap();
    let boot_rom = fs::read(&cli.boot_rom).unwrap();

    let gb = match qgb::GameBoy::new(&rom, &boot_rom) {
        Ok(gb) => gb,
        Err(qgb::BootError::BootRomError(e)) => {
            eprintln!("'{}': {}", cli.boot_rom.display(), e);
            return;
        }
        Err(qgb::BootError::RomError(e)) => {
            eprintln!("'{}': {}", cli.boot_rom.display(), e);
            return;
        }
    };

    let mut debugger = Debugger::new(gb);
    debugger.start();
}

fn init_logger() {
    if let Err(err) = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .without_time()
                .with_span_events(fmt::format::FmtSpan::ACTIVE)
                .pretty(),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .try_init()
    {
        eprintln!(
            "an error occurred initializing the tracing library: {}",
            err
        );
    }
}

struct Debugger {
    gb: qgb::GameBoy,
}

#[derive(Debug)]
enum Command {
    Dump,
    Step,
    Run(usize),
    Exit,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err("Invalid command".into());
        }
        let cmd: Vec<String> = s
            .trim()
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if cmd.is_empty() {
            return Err("No command specified".into());
        }
        match cmd[0].as_str() {
            "dump" | "d" => Ok(Command::Dump),
            "step" | "s" => Ok(Command::Step),
            "run" | "r" => {
                if cmd.len() == 1 {
                    Ok(Command::Run(0))
                } else if cmd.len() == 2 {
                    if let Ok(step_size) = cmd[1].parse() {
                        Ok(Command::Run(step_size))
                    } else {
                        Err("Unrecognized step size".into())
                    }
                } else {
                    Err("Incorrect number of arguments specified".into())
                }
            }
            "exit" | "quit" | "e" | "q" => Ok(Command::Exit),
            _ => Err("Invalid command".into()),
        }
    }
}

impl Debugger {
    pub fn new(gb: qgb::GameBoy) -> Self {
        Self { gb }
    }

    pub fn start(&mut self) {
        println!("Initializing debugger");
        loop {
            self.prologue();
            print!("qgb> ");
            io::stdout().flush().unwrap();

            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let cmd = match buffer.parse() {
                Ok(cmd) => cmd,
                Err(msg) => {
                    println!("{}", msg);
                    continue;
                }
            };

            match cmd {
                Command::Dump => println!("{:#X?}", self.gb.state()),
                Command::Step => {
                    self.gb.step();
                }
                Command::Run(mut steps) => {
                    if steps == 0 {
                        steps = usize::MAX;
                    }
                    for step in 0..steps {
                        self.gb.step();
                        if step != steps - 1 {
                            self.prologue();
                        }
                    }
                }
                Command::Exit => return,
            }
        }
    }

    fn prologue(&mut self) {
        if let Some(cpu_state) = &self.gb.state().cpu {
            println!("{:X?}", cpu_state.instructions[0]);
        }
    }
}
