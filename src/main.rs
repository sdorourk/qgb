mod debugger;

use std::{fs, path::PathBuf, sync::mpsc::channel, time};

use clap::Parser;

use debugger::Message;
use qgb::{Color, TCycles, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const RGBA_WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
const RGBA_LIGHT_GRAY: [u8; 4] = [0x66, 0x66, 0x66, 0xFF];
const RGBA_DARK_GRAY: [u8; 4] = [0xB2, 0xB2, 0xB2, 0xFF];
const RGBA_BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];

const DEFAULT_SCREEN_SCALE: u32 = 5;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// ROM program to run
    program: PathBuf,
    /// Boot ROM
    #[arg(short, long)]
    boot_rom: PathBuf,
    /// Console logger (for comparing logs with other emulators)
    #[arg(short, long)]
    console_log: bool,
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

    if let Err(msg) = run(gb, cli.console_log) {
        eprintln!("A fatal error occurred: {}", msg);
    }
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

const CYCLES_PER_SEC: TCycles = 4 * 1024 * 1024;
const FRAMES_PER_SEC: TCycles = 60;
const CYCLES_PER_FRAME: TCycles = CYCLES_PER_SEC / FRAMES_PER_SEC;

#[derive(Debug)]
enum EmulatorRunState {
    Pause,
    Run,
    Step,
}

fn run(mut gb: qgb::GameBoy, console_log: bool) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Required to avoid excessive conversions
    const HEIGHT: u32 = DISPLAY_HEIGHT as u32;
    const WIDTH: u32 = DISPLAY_WIDTH as u32;

    // Initialize the window
    let window = video_subsystem
        .window(
            "Game Boy Emulator",
            WIDTH * DEFAULT_SCREEN_SCALE,
            HEIGHT * DEFAULT_SCREEN_SCALE,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    canvas
        .set_logical_size(WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA32, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let (msg_sender, msg_receiver) = channel::<Message>();
    let mut debugger = debugger::Debugger::new(msg_sender, gb.state());
    let mut console_logger = DefaultConsoleLogger::new(console_log, false);
    debugger.update(gb.state());
    console_logger.print_log(&mut gb);
    let mut run_state = EmulatorRunState::Pause;
    let mut cycle_count: TCycles = 0;
    let mut clock = Clock::new(time::Duration::from_secs_f64(0.016));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = key_map(key) {
                        gb.button_pressed(button);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = key_map(key) {
                        gb.button_released(button);
                    }
                }
                _ => {}
            }
        }

        debugger.handle_events();
        match msg_receiver.try_recv() {
            Ok(Message::Pause) => run_state = EmulatorRunState::Pause,
            Ok(Message::Run) => run_state = EmulatorRunState::Run,
            Ok(Message::Step) => run_state = EmulatorRunState::Step,
            Ok(Message::Quit) => break,
            Err(_) => {}
        }

        match run_state {
            EmulatorRunState::Pause => {
                cycle_count = 0;
            }
            EmulatorRunState::Run => {
                let breakpoints = debugger.breakpoints();
                cycle_count += CYCLES_PER_FRAME;
                while cycle_count > 0 {
                    cycle_count -= gb.step();
                    console_logger.print_log(&mut gb);
                    if breakpoints.contains(&gb.pc()) {
                        run_state = EmulatorRunState::Pause;
                        break;
                    }
                }
                debugger.update(gb.state());
            }
            EmulatorRunState::Step => {
                cycle_count = 0;
                gb.step();
                console_logger.print_log(&mut gb);
                run_state = EmulatorRunState::Pause;
                debugger.update(gb.state());
            }
        }

        let pixels = colors_to_rgba32(&gb.screen());
        texture.with_lock(None, |buffer: &mut [u8], _: usize| {
            buffer.copy_from_slice(&pixels);
        })?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        clock.wait();
    }

    Ok(())
}

fn colors_to_rgba32(colors: &[Color]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(colors.len() * 4);
    for color in colors {
        rgba.extend_from_slice(match color {
            Color::White => &RGBA_WHITE,
            Color::LightGray => &RGBA_LIGHT_GRAY,
            Color::DarkGray => &RGBA_DARK_GRAY,
            Color::Black => &RGBA_BLACK,
        });
    }
    rgba
}

fn key_map(key: Keycode) -> Option<qgb::JoypadButton> {
    match key {
        Keycode::Up | Keycode::W => Some(qgb::JoypadButton::Up),
        Keycode::Down | Keycode::S => Some(qgb::JoypadButton::Down),
        Keycode::Right | Keycode::D => Some(qgb::JoypadButton::Right),
        Keycode::Left | Keycode::A => Some(qgb::JoypadButton::Left),
        Keycode::Space => Some(qgb::JoypadButton::Select),
        Keycode::Return => Some(qgb::JoypadButton::Start),
        Keycode::Z | Keycode::J => Some(qgb::JoypadButton::A),
        Keycode::X | Keycode::K => Some(qgb::JoypadButton::B),
        _ => None,
    }
}

struct Clock {
    duration: time::Duration,
    start_instant: time::Instant,
}

impl Clock {
    pub fn new(duration: time::Duration) -> Self {
        Self {
            duration,
            start_instant: time::Instant::now(),
        }
    }

    pub fn start(&mut self) {
        self.start_instant = time::Instant::now();
    }

    pub fn wait(&mut self) {
        let duration = time::Instant::now().duration_since(self.start_instant);
        match duration.cmp(&self.duration) {
            std::cmp::Ordering::Less => {
                std::thread::sleep(self.duration - duration);
            }
            std::cmp::Ordering::Greater => {
                tracing::warn!(target: "emulator", "slow frame: frame took {} μs longer than expected", (duration - self.duration).as_micros())
            }
            std::cmp::Ordering::Equal => {}
        }
        self.start();
    }
}

trait ConsoleLogger {
    fn print_log(&mut self, gb: &mut qgb::GameBoy);
}

#[derive(Debug)]
struct DefaultConsoleLogger {
    enabled: bool,
    log_boot_rom: bool,
    boot_rom_ended: bool,
}

impl DefaultConsoleLogger {
    pub fn new(enabled: bool, log_boot_rom: bool) -> Self {
        Self {
            enabled,
            log_boot_rom,
            boot_rom_ended: false,
        }
    }
}

impl ConsoleLogger for DefaultConsoleLogger {
    fn print_log(&mut self, gb: &mut qgb::GameBoy) {
        if self.enabled {
            let state = gb.state();
            if let Some(cpu_state) = &state.cpu {
                if cpu_state.pc == 0x101 {
                    self.boot_rom_ended = true;
                }
                if self.log_boot_rom || self.boot_rom_ended {
                    let mut memory = Vec::new();
                    for i in 0..4 {
                        memory.extend_from_slice(&cpu_state.instructions[i].bytes[..]);
                    }
                    println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})", cpu_state.a, cpu_state.f, cpu_state.b, cpu_state.c, cpu_state.d, cpu_state.e, cpu_state.h, cpu_state.l, cpu_state.sp, cpu_state.pc, memory[0], memory[1], memory[2], memory[3]);
                }
            }
        }
    }
}
