mod widgets;

use std::sync::mpsc::Sender;

use fltk::{
    app::{self},
    browser,
    button::{Button, CheckButton},
    enums::CallbackTrigger,
    frame::Frame,
    group::{Flex, Tabs},
    input,
    prelude::*,
    text::TextDisplay,
    window::DoubleWindow,
};

use self::widgets::{
    EmitButton, InstructionBrowser, MemoryTable, RegisterDisplay, WideRegisterDisplay,
};

const WINDOW_WIDTH: i32 = 1000;
const WINDOW_HEIGHT: i32 = 800;
const PADDING: i32 = 10;
const MARGIN: i32 = 10;
const COMMAND_COLUMN_WIDTH: i32 = 400;
const BUTTON_HEIGHT: i32 = 35;
const LABEL_HEIGHT: i32 = BUTTON_HEIGHT;
const BREAKPOINT_BROWSER_HEIGHT: i32 = 3 * BUTTON_HEIGHT;

#[derive(Debug)]
pub struct Debugger {
    disassembly: InstructionBrowser,
    reg_a: RegisterDisplay,
    reg_b: RegisterDisplay,
    reg_c: RegisterDisplay,
    reg_d: RegisterDisplay,
    reg_e: RegisterDisplay,
    reg_f: RegisterDisplay,
    reg_h: RegisterDisplay,
    reg_l: RegisterDisplay,
    cpu_z_flag: CheckButton,
    cpu_n_flag: CheckButton,
    cpu_h_flag: CheckButton,
    cpu_c_flag: CheckButton,
    reg_pc: WideRegisterDisplay,
    reg_sp: WideRegisterDisplay,
    rom_table: MemoryTable,
    external_ram_table: MemoryTable,
    wram_table: MemoryTable,
    hram_table: MemoryTable,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Pause,
    Run,
    Step,
    Quit,
}

impl Debugger {
    pub fn new(msg_sender: Sender<Message>) -> Self {
        let _app = app::App::default();

        let disassembly;
        let reg_a;
        let reg_b;
        let reg_c;
        let reg_d;
        let reg_e;
        let reg_f;
        let reg_h;
        let reg_l;
        let cpu_z_flag;
        let cpu_n_flag;
        let cpu_h_flag;
        let cpu_c_flag;
        let reg_pc;
        let reg_sp;
        let rom_table;
        let external_ram_table;
        let wram_table;
        let hram_table;

        let mut window = DoubleWindow::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_label("Debugger");

        let mut row = Flex::default_fill().row();
        {
            let mut col = Flex::default_fill().column();
            {
                {
                    let mut row = Flex::default_fill().row();
                    let _run = EmitButton::new("Run", msg_sender.clone(), Message::Run);
                    let _pause = EmitButton::new("Pause", msg_sender.clone(), Message::Pause);
                    let _step = EmitButton::new("Step", msg_sender.clone(), Message::Step);
                    row.end();
                    row.set_pad(PADDING);
                    col.fixed(&row, BUTTON_HEIGHT);
                }

                disassembly = InstructionBrowser::new();

                let breakpoint_label = Frame::default().with_label("Breakpoints");
                col.fixed(&breakpoint_label, LABEL_HEIGHT);

                let breakpoint_list = browser::SelectBrowser::default();
                {
                    let row = Flex::default_fill().row();
                    let _breakpoint_input = input::Input::default();
                    let _add_breakpoint = Button::default().with_label("Add");
                    let _delete_breakpoint = Button::default().with_label("Delete");
                    row.end();
                    col.fixed(&row, BUTTON_HEIGHT);
                }
                col.fixed(&breakpoint_list, BREAKPOINT_BROWSER_HEIGHT);
            }
            col.end();
            col.set_pad(PADDING);
            col.set_margin(MARGIN);
            row.fixed(&col, COMMAND_COLUMN_WIDTH);
        }
        {
            let mut col = Flex::default_fill().column();
            {
                let mut component_tabs = Tabs::default_fill();
                {
                    let mut row = Flex::default_fill().row().with_label("Overview\t");
                    {
                        let col = Flex::default_fill().column();
                        {
                            let row = Flex::default_fill().row();
                            {
                                let mut col = Flex::default_fill().column();
                                let cpu_register_label =
                                    Frame::default().with_label("CPU Registers");
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("A:");
                                    reg_a = RegisterDisplay::default();
                                    Frame::default().with_label("F:");
                                    reg_f = RegisterDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("B:");
                                    reg_b = RegisterDisplay::default();
                                    Frame::default().with_label("C:");
                                    reg_c = RegisterDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("D:");
                                    reg_d = RegisterDisplay::default();
                                    Frame::default().with_label("E:");
                                    reg_e = RegisterDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("H:");
                                    reg_h = RegisterDisplay::default();
                                    Frame::default().with_label("L:");
                                    reg_l = RegisterDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("Flags:");
                                    cpu_z_flag = CheckButton::default_fill().with_label("Z");
                                    cpu_n_flag = CheckButton::default_fill().with_label("N");
                                    cpu_h_flag = CheckButton::default_fill().with_label("H");
                                    cpu_c_flag = CheckButton::default_fill().with_label("C");
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                Frame::default(); // Filler
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("PC:");
                                    reg_pc = WideRegisterDisplay::default();
                                    Frame::default().with_label("SP:");
                                    reg_sp = WideRegisterDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                Frame::default(); // Filler
                                col.end();
                                col.fixed(&cpu_register_label, BUTTON_HEIGHT);
                                col.set_pad(PADDING);
                                col.set_margin(MARGIN);
                            }
                            {
                                let mut col = Flex::default_fill().column();
                                let cartridge_label = Frame::default().with_label("Cartridge");
                                let _cartridge_state = browser::Browser::default();
                                let serial_label =
                                    Frame::default().with_label("Serial Data Output");
                                let _serial_data = TextDisplay::default();
                                col.end();
                                col.fixed(&cartridge_label, BUTTON_HEIGHT);
                                col.fixed(&serial_label, BUTTON_HEIGHT);
                                col.set_pad(PADDING);
                                col.set_margin(MARGIN);
                            }

                            row.end();
                        }

                        {
                            let mut memory_tabs = Tabs::default_fill();
                            {
                                let mut row = Flex::default_fill().row().with_label("ROM\t");
                                rom_table = widgets::MemoryTable::new(&mut row);
                                row.end();
                                row.set_margin(MARGIN);
                            }
                            {
                                let mut row =
                                    Flex::default_fill().row().with_label("External RAM\t");
                                external_ram_table = widgets::MemoryTable::new(&mut row);
                                row.end();
                                row.set_margin(MARGIN);
                            }
                            {
                                let mut row = Flex::default_fill().row().with_label("WRAM\t");
                                wram_table = widgets::MemoryTable::new(&mut row);
                                row.end();
                                row.set_margin(MARGIN);
                            }
                            {
                                let mut row = Flex::default_fill().row().with_label("HRAM\t");
                                hram_table = widgets::MemoryTable::new(&mut row);
                                row.end();
                                row.set_margin(MARGIN);
                            }
                            memory_tabs.end();
                            memory_tabs.visible_focus(false);
                            memory_tabs.auto_layout();
                        }
                        col.end();
                    }
                    row.end();
                    row.set_pad(PADDING);
                    row.set_margin(MARGIN);
                }
                {
                    let row = Flex::default_fill().row().with_label("PPU\t");
                    Frame::default().with_label("No data available");
                    row.end();
                }
                {
                    let row = Flex::default_fill().row().with_label("APU\t");
                    Frame::default().with_label("No data available");
                    row.end();
                }
                component_tabs.end();
                component_tabs.visible_focus(false);
                component_tabs.auto_layout();
            }
            col.end();
            col.set_pad(PADDING);
            col.set_margin(MARGIN);
        }

        row.end();
        row.set_pad(PADDING);
        row.set_margin(MARGIN);

        window.end();
        window.set_trigger(CallbackTrigger::Closed);
        window.set_callback(move |_| {
            _ = msg_sender.send(Message::Quit);
        });
        window.resizable(&row);
        window.size_range(WINDOW_WIDTH, WINDOW_HEIGHT, 0, 0);
        window.show();

        Self {
            disassembly,
            reg_a,
            reg_b,
            reg_c,
            reg_d,
            reg_e,
            reg_f,
            reg_h,
            reg_l,
            cpu_z_flag,
            cpu_n_flag,
            cpu_h_flag,
            cpu_c_flag,
            reg_pc,
            reg_sp,
            rom_table,
            external_ram_table,
            wram_table,
            hram_table,
        }
    }

    pub fn handle_events(&mut self) {
        app::check();
        app::redraw();
    }

    pub fn update(&mut self, state: &qgb::State) {
        // CPU state
        if let Some(cpu_state) = &state.cpu {
            self.disassembly.update(&cpu_state.instructions);
            self.reg_a.update(cpu_state.a);
            self.reg_b.update(cpu_state.b);
            self.reg_c.update(cpu_state.c);
            self.reg_d.update(cpu_state.d);
            self.reg_e.update(cpu_state.e);
            self.reg_f.update(cpu_state.f);
            self.reg_h.update(cpu_state.h);
            self.reg_l.update(cpu_state.l);
            self.cpu_z_flag.set(cpu_state.z_flag);
            self.cpu_n_flag.set(cpu_state.n_flag);
            self.cpu_h_flag.set(cpu_state.h_flag);
            self.cpu_c_flag.set(cpu_state.c_flag);
            self.reg_pc.update(cpu_state.pc);
            self.reg_sp.update(cpu_state.sp);
        }
    }
}
