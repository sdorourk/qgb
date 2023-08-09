use std::sync::mpsc::{self};

use fltk::{
    app::{self},
    browser,
    button::{Button, CheckButton},
    enums::{CallbackTrigger, Color},
    frame::Frame,
    group::{Flex, Tabs},
    input,
    prelude::*,
    text::TextDisplay,
    window::DoubleWindow,
};
use fltk_table::{SmartTable, TableOpts};

const WINDOW_WIDTH: i32 = 1000;
const WINDOW_HEIGHT: i32 = 800;
const PADDING: i32 = 10;
const MARGIN: i32 = 10;
const COMMAND_COLUMN_WIDTH: i32 = 400;
const BUTTON_HEIGHT: i32 = 35;
const LABEL_HEIGHT: i32 = BUTTON_HEIGHT;
const BREAKPOINT_BROWSER_HEIGHT: i32 = 3 * BUTTON_HEIGHT;
const MEMORY_TABLE_NUMBER_OF_COLUMNS: i32 = 16;
const MEMORY_TABLE_ROW_HEADER_WIDTH: i32 = 55;
const MEMORY_TABLE_ROW_WIDTH_OFFSET: i32 = 37;

#[derive(Debug)]
pub struct Debugger {
    msg_rx: mpsc::Receiver<Message>,
    request_close: bool,
}

#[derive(Debug)]
enum Message {
    Pause,
    Run,
    Step,
    Quit,
}

impl Debugger {
    pub fn new(_state: &qgb::State) -> Self {
        let _app = app::App::default();
        let (msg_tx, msg_rx) = mpsc::channel::<Message>();

        let mut window = DoubleWindow::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_label("Debugger");

        let mut row = Flex::default_fill().row();
        {
            let mut col = Flex::default_fill().column();
            {
                {
                    let mut row = Flex::default_fill().row();
                    let mut run = Button::default().with_label("Run");
                    let mut pause = Button::default().with_label("Pause");
                    let mut step = Button::default().with_label("Step");
                    row.end();
                    row.set_pad(PADDING);
                    col.fixed(&row, BUTTON_HEIGHT);

                    let msg_tx_clone1 = msg_tx.clone();
                    let msg_tx_clone2 = msg_tx.clone();
                    let msg_tx_clone3 = msg_tx.clone();
                    run.set_callback(move |_| {
                        _ = msg_tx_clone1.send(Message::Run);
                    });
                    pause.set_callback(move |_| {
                        _ = msg_tx_clone2.send(Message::Pause);
                    });
                    step.set_callback(move |_| {
                        _ = msg_tx_clone3.send(Message::Step);
                    });
                }

                let _disassembly = browser::Browser::default();

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
                                    let _reg_a = TextDisplay::default();
                                    Frame::default().with_label("F:");
                                    let _reg_f = TextDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("B:");
                                    let _reg_b = TextDisplay::default();
                                    Frame::default().with_label("C:");
                                    let _reg_c = TextDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("D:");
                                    let _reg_d = TextDisplay::default();
                                    Frame::default().with_label("E:");
                                    let _reg_e = TextDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("H:");
                                    let _reg_h = TextDisplay::default();
                                    Frame::default().with_label("L:");
                                    let _reg_l = TextDisplay::default();
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("Flags:");
                                    let _cpu_z_flag = CheckButton::default_fill().with_label("Z");
                                    let _cpu_n_flag = CheckButton::default_fill().with_label("N");
                                    let _cpu_h_flag = CheckButton::default_fill().with_label("H");
                                    let _cpu_c_flag = CheckButton::default_fill().with_label("C");
                                    row.end();
                                    col.fixed(&row, BUTTON_HEIGHT);
                                }
                                Frame::default(); // Filler
                                {
                                    let row = Flex::default_fill().row();
                                    Frame::default().with_label("PC:");
                                    let _reg_pc = TextDisplay::default();
                                    Frame::default().with_label("SP:");
                                    let _reg_sp = TextDisplay::default();
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
                                let mut rom_table = SmartTable::default().with_opts(TableOpts {
                                    rows: 0x4000 / MEMORY_TABLE_NUMBER_OF_COLUMNS,
                                    cols: MEMORY_TABLE_NUMBER_OF_COLUMNS,
                                    editable: false,
                                    cell_border_color: Color::BackGround.lighter(),
                                    ..Default::default()
                                });
                                rom_table.set_row_header_width(MEMORY_TABLE_ROW_HEADER_WIDTH);
                                for i in 0..MEMORY_TABLE_NUMBER_OF_COLUMNS {
                                    rom_table.set_col_header_value(i, &format!("{:01X}", i));
                                }
                                for i in 0..rom_table.row_count() {
                                    rom_table.set_row_header_value(
                                        i,
                                        &format!("{:04X}", i * MEMORY_TABLE_NUMBER_OF_COLUMNS),
                                    );
                                }
                                row.end();
                                row.set_margin(MARGIN);
                                row.resize_callback(move |_, _, _, w, _| {
                                    rom_table.set_col_width_all(
                                        (w - MEMORY_TABLE_ROW_HEADER_WIDTH
                                            - MEMORY_TABLE_ROW_WIDTH_OFFSET)
                                            / MEMORY_TABLE_NUMBER_OF_COLUMNS,
                                    );
                                });
                            }
                            {
                                let row = Flex::default_fill().row().with_label("External RAM\t");
                                Frame::default().with_label("External RAM here");
                                row.end();
                            }
                            {
                                let row = Flex::default_fill().row().with_label("WRAM\t");
                                Frame::default().with_label("WRAM here");
                                row.end();
                            }
                            {
                                let row = Flex::default_fill().row().with_label("HRAM\t");
                                Frame::default().with_label("HRAM here");
                                row.end();
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
            _ = msg_tx.send(Message::Quit);
        });
        window.resizable(&row);
        window.size_range(WINDOW_WIDTH, WINDOW_HEIGHT, 0, 0);
        window.show();

        Self {
            msg_rx,
            request_close: false,
        }
    }

    pub fn handle_events(&mut self) {
        app::check();
        if let Ok(msg) = self.msg_rx.try_recv() {
            match msg {
                Message::Pause => println!("Pause"),
                Message::Run => println!("Run"),
                Message::Step => println!("Step"),
                Message::Quit => self.request_close = true,
            }
        }
    }

    pub fn request_close(&self) -> bool {
        self.request_close
    }
}
