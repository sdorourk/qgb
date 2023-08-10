use std::ops::{Deref, DerefMut};

use fltk::{enums::Color, group::Flex, prelude::*};
use fltk_table::{SmartTable, TableOpts};

const MEMORY_TABLE_NUMBER_OF_COLUMNS: i32 = 16;
const MEMORY_TABLE_ROW_HEADER_WIDTH: i32 = 55;
const MEMORY_TABLE_ROW_WIDTH_OFFSET: i32 = 37;

pub struct MemoryTable {
    table: SmartTable,
    data_size: i32,
}

impl MemoryTable {
    pub fn new(flex_row: &mut Flex, data_size: i32) -> Self {
        assert!(data_size > 0);
        assert_eq!(data_size % MEMORY_TABLE_NUMBER_OF_COLUMNS, 0);

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
        assert_eq!(data_size, self.data_size);

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
