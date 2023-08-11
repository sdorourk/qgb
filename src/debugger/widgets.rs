use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;

use fltk::browser;
use fltk::text::{TextBuffer, TextDisplay};
use fltk::{button::Button, enums::Color, group::Flex, prelude::*};
use fltk_table::{SmartTable, TableOpts};
use qgb::state::InstructionInfo;

const MEMORY_TABLE_NUMBER_OF_COLUMNS: i32 = 16;
const MEMORY_TABLE_ROW_HEADER_WIDTH: i32 = 55;
const MEMORY_TABLE_ROW_WIDTH_OFFSET: i32 = 37;

#[derive(Debug)]
pub struct MemoryTable {
    table: SmartTable,
    data_size: i32,
}

impl MemoryTable {
    pub fn new(flex_row: &mut Flex) -> Self {
        // The actual value for `data_size` will be specified once `update()` is called
        // for the first time
        let data_size = MEMORY_TABLE_NUMBER_OF_COLUMNS * 20;

        let table = SmartTable::default().with_opts(TableOpts {
            rows: data_size / MEMORY_TABLE_NUMBER_OF_COLUMNS,
            cols: MEMORY_TABLE_NUMBER_OF_COLUMNS,
            editable: false,
            cell_border_color: Color::BackGround.lighter(),
            ..Default::default()
        });

        flex_row.resize_callback({
            let mut table = table.clone();
            move |_, _, _, w, _| {
                table.set_col_width_all(
                    (w - MEMORY_TABLE_ROW_HEADER_WIDTH - MEMORY_TABLE_ROW_WIDTH_OFFSET)
                        / MEMORY_TABLE_NUMBER_OF_COLUMNS,
                );
            }
        });

        let mut memory_table = Self { table, data_size };
        memory_table.update_data_size(data_size);
        memory_table
    }

    fn update_data_size(&mut self, data_size: i32) {
        assert!(data_size > 0);
        assert_eq!(data_size % MEMORY_TABLE_NUMBER_OF_COLUMNS, 0);

        self.data_size = data_size;
        self.table
            .set_rows(data_size / MEMORY_TABLE_NUMBER_OF_COLUMNS);
        self.table
            .set_row_header_width(MEMORY_TABLE_ROW_HEADER_WIDTH);
        for i in 0..MEMORY_TABLE_NUMBER_OF_COLUMNS {
            self.table.set_col_header_value(i, &format!("{:01X}", i));
        }
        for i in 0..self.table.row_count() {
            self.table
                .set_row_header_value(i, &format!("{:04X}", i * MEMORY_TABLE_NUMBER_OF_COLUMNS));
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        assert!(!data.is_empty());
        assert!(i32::try_from(data.len()).is_ok());

        let data_size = i32::try_from(data.len()).unwrap();

        if data_size != self.data_size {
            self.update_data_size(data_size);
        }

        for i in 0..self.table.row_count() {
            for j in 0..MEMORY_TABLE_NUMBER_OF_COLUMNS {
                let index =
                    usize::try_from(i * MEMORY_TABLE_NUMBER_OF_COLUMNS + j).unwrap_or_default();
                self.table
                    .set_cell_value(i, j, &format!("{:02X}", data[index]));
            }
        }
    }
}

impl Deref for MemoryTable {
    type Target = SmartTable;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl DerefMut for MemoryTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}

#[derive(Debug)]
pub struct EmitButton {
    button: Button,
}

impl EmitButton {
    pub fn new<T>(label: &str, sender: Sender<T>, message: T) -> Self
    where
        T: 'static + Copy,
    {
        let mut button = Button::default().with_label(label);
        button.set_callback(move |_| {
            _ = sender.send(message);
        });
        button.visible_focus(false);
        Self { button }
    }
}

impl Deref for EmitButton {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.button
    }
}

impl DerefMut for EmitButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.button
    }
}

#[derive(Debug)]
pub struct InstructionBrowser {
    browser: browser::Browser,
}

impl InstructionBrowser {
    pub fn new() -> Self {
        let mut browser = browser::Browser::default();
        browser.set_column_char('\t');
        browser.set_column_widths(&[50, 30, 30, 50, 300]);
        Self { browser }
    }

    pub fn update(&mut self, instructions: &[InstructionInfo]) {
        self.browser.clear();
        for instr in instructions {
            let mut bytes: Vec<String> = instr.bytes.iter().map(|b| format!("{:02X}", b)).collect();
            if bytes.len() < 3 {
                bytes.extend_from_slice(&["".into(), "".into(), "".into()]);
            }

            let str_instr = format!(
                "@b{:04X}:\t@C42 {}\t@C42 {}\t@C42 {}\t@C4 {}",
                instr.address, bytes[0], bytes[1], bytes[2], instr.display
            );
            self.browser.add(&str_instr);
        }
    }
}

impl Deref for InstructionBrowser {
    type Target = browser::Browser;

    fn deref(&self) -> &Self::Target {
        &self.browser
    }
}

impl DerefMut for InstructionBrowser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.browser
    }
}

#[derive(Debug, Default)]
pub struct RegisterDisplay {
    display: TextDisplay,
}

impl RegisterDisplay {
    pub fn update(&mut self, value: u8) {
        let format_str = format!("{:02X}", value);
        match self.display.buffer() {
            Some(mut buffer) => {
                buffer.set_text(&format_str);
            }
            None => {
                let mut buffer = TextBuffer::default();
                buffer.set_text(&format_str);
                self.display.set_buffer(buffer);
            }
        }
    }
}

impl Deref for RegisterDisplay {
    type Target = TextDisplay;

    fn deref(&self) -> &Self::Target {
        &self.display
    }
}

impl DerefMut for RegisterDisplay {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.display
    }
}

#[derive(Debug, Default)]
pub struct WideRegisterDisplay {
    display: TextDisplay,
}

impl WideRegisterDisplay {
    pub fn update(&mut self, value: u16) {
        let format_str = format!("{:04X}", value);
        match self.display.buffer() {
            Some(mut buffer) => {
                buffer.set_text(&format_str);
            }
            None => {
                let mut buffer = TextBuffer::default();
                buffer.set_text(&format_str);
                self.display.set_buffer(buffer);
            }
        }
    }
}

impl Deref for WideRegisterDisplay {
    type Target = TextDisplay;

    fn deref(&self) -> &Self::Target {
        &self.display
    }
}

impl DerefMut for WideRegisterDisplay {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.display
    }
}
