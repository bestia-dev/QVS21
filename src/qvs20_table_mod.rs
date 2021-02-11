// qvs20_table_mod

//! The table structure is very flexible, because it is defined in runtime.
//! A lot of times the table is used just as an intermediary,
//! and don't need a fixed Rust struct in compile time.
//! It means that sometimes a change in the table does not dictate change in source code and compiling.

use crate::qvs20_reader_mod::*;
use crate::qvs20_table_rows_mod::*;
use crate::qvs20_table_schema_mod::*;
use crate::qvs20_writer_mod::*;

/// The Table type contains TableSchema and TableRows.
///
/// It allows run-time typed presentation of tabular data.
/// This is great when we don't know at compile time the schema of the data.
/// It can be read and write into qvs20 utf8 string format.
/// # Example
///
/// ```rust
/// use std::fs;
/// use unwrap::unwrap;
/// use qvs20::*;
///
/// let text = unwrap!(fs::read_to_string("sample_data/write/country_population.qvs20"));
/// let table = unwrap!(Table::from_qvs20_str_with_schema(&text));
/// println!("{:#?}", table);
///
/// ```
#[derive(Clone, Debug, Default)]
pub struct Table {
    /// schema
    pub schema: TableSchema,
    /// rows of data
    pub table_rows: TableRows,
}

impl Table {
    /// create a table object in memory from a qvs20 format string
    pub fn from_qvs20_str_with_schema(input: &str) -> Result<Table, Qvs20Error> {
        // the input is String to ensure it is well-formed utf8
        // internally use [u8] for performance
        let input = input.as_bytes();
        let mut rdr = ReaderForQvs20::new(input);

        let mut schema = TableSchema::default();
        schema.read_schema(&mut rdr)?;

        let mut table_rows = TableRows::default();
        table_rows.row_delimiter = schema.row_delimiter;
        table_rows.active_row = 0;
        table_rows.active_column = 0;
        table_rows.append_data_rows(&mut rdr, &schema)?;

        let table = Table { schema, table_rows };
        //return
        Ok(table)
    }

    /// write to String
    pub fn write_table(&self) -> String {
        let mut wrt = WriterForQvs20::new();
        self.schema.write_schema_to_writer(&mut wrt, false);
        self.table_rows.write_table_rows_to_writer(&mut wrt);
        //return
        wrt.return_and_finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use unwrap::unwrap;

    #[test]
    pub fn t01_string_table() {
        let s = r"[T][table_name][table_description]
[String][String]
[][]
[blue][red]
[name][description]
[name_1][description_1]
[name_2][description_2]
[name_3][description_3]
";
        let table = unwrap!(Table::from_qvs20_str_with_schema(&s));
        assert_eq!(format!("{:?}",table),"Table { schema: TableSchema { table_name: \"table_name\", table_description: \"table_description\", data_types: [String, String], sub_table_schemas: [None, None], additional_properties: [\"blue\", \"red\"], column_names: [\"name\", \"description\"], active_row: 4, active_column: 2, row_delimiter: 10, sub_table_row_delimiter: 0 }, table_rows: TableRows { rows: [Row { values: [String(\"name_1\"), String(\"description_1\")] }, Row { values: [String(\"name_2\"), String(\"description_2\")] }, Row { values: [String(\"name_3\"), String(\"description_3\")] }], table_name: \"\", active_row: 3, active_column: 0, row_delimiter: 10, sub_table_row_delimiter: 0 } }");
    }
    #[test]
    pub fn t02_all_wrong_in_data_rows() {
        let pre_string = "[T][table name][description]\n[String][String][String]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[string1";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 0 Last bracket is missing. ./file.qvs20:5:2"
        );

        let data = "[string1][11]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 0 has 2 columns differ from schema 3 columns"
        );

        let data = "[string1][11][11.11]";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 0 Last row delimiter is missing. ./file.qvs20:5:21"
        );

        let data = "[string1][11][11.11]1";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: data row 0 wrong row delimiter:49 instead of 10"
        );

        let data = "[string1][11][11.11]\n\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 0 The row delimiter has more than 1 byte. ./file.qvs20:5:21"
        );

        // rows of data active_row 1
        let pre_string = format!("{}{}", pre_string, "[string1][11][11.11]\n");

        let data = "[string2";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 1 Last bracket is missing. ./file.qvs20:6:2"
        );

        let data = "[string2][22]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 1 has 2 columns differ from schema 3 columns"
        );

        let data = "[string2][22][22.22]";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 1 Last row delimiter is missing. ./file.qvs20:6:21"
        );

        let data = "[string2][22][22.22]1";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: data row 1 wrong row delimiter:49 instead of 10"
        );

        let data = "[string2][22][22.22]\n\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 1 The row delimiter has more than 1 byte. ./file.qvs20:6:21"
        );

        // rows of data active_row 2
        let pre_string = format!(
            "{}{}",
            pre_string, "[string1][11][11.11]\n[string2][22][22.22]\n"
        );

        // missing escape char
        let data = "[str]ing1][11][11.11]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: row 3 The row delimiter has more than 1 byte. ./file.qvs20:8:6"
        );
    }
    #[test]
    pub fn t03_data_type_integer() {
        let pre_string = "[T][table name][description]\n[Integer][Integer][Integer]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[11][22][a]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to integer. invalid digit found in string. row 0 col 2"
        );

        let data = "[-11][-22][1.1]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to integer. invalid digit found in string. row 0 col 2"
        );

        let data = "[+11][+22][1,2]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to integer. invalid digit found in string. row 0 col 2"
        );

        let data = "[+11][-22][0]\n";
        let s = format!("{}{}", pre_string, data);
        let table = unwrap!(Table::from_qvs20_str_with_schema(&s));
        assert_eq!(format!("{:?}", table), "Table { schema: TableSchema { table_name: \"table name\", table_description: \"description\", data_types: [Integer, Integer, Integer], sub_table_schemas: [None, None, None], additional_properties: [\"prop1\", \"prop2\", \"prop3\"], column_names: [\"name1\", \"name2\", \"name3\"], active_row: 4, active_column: 3, row_delimiter: 10, sub_table_row_delimiter: 0 }, table_rows: TableRows { rows: [Row { values: [Integer(11), Integer(-22), Integer(0)] }], table_name: \"\", active_row: 1, active_column: 0, row_delimiter: 10, sub_table_row_delimiter: 0 } }");
    }
    #[test]
    pub fn t04_data_type_decimal() {
        let pre_string = "[T][table name][description]\n[Decimal][Decimal][Decimal]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[11.1][22.22][a]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to decimal. Invalid decimal: invalid character. row 0 col 2"
        );

        let data = "[-11.1][-22.22][1,1]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to decimal. Invalid decimal: unknown character. row 0 col 2"
        );

        let data = "[+11.1][+22.22][12e2]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to decimal. Invalid decimal: invalid character. row 0 col 2"
        );

        let data = "[+11.1][-22.22][0]\n";
        let s = format!("{}{}", pre_string, data);
        let table = unwrap!(Table::from_qvs20_str_with_schema(&s));
        assert_eq!(format!("{:?}", table), "Table { schema: TableSchema { table_name: \"table name\", table_description: \"description\", data_types: [Decimal, Decimal, Decimal], sub_table_schemas: [None, None, None], additional_properties: [\"prop1\", \"prop2\", \"prop3\"], column_names: [\"name1\", \"name2\", \"name3\"], active_row: 4, active_column: 3, row_delimiter: 10, sub_table_row_delimiter: 0 }, table_rows: TableRows { rows: [Row { values: [Decimal(11.1), Decimal(-22.22), Decimal(0)] }], table_name: \"\", active_row: 1, active_column: 0, row_delimiter: 10, sub_table_row_delimiter: 0 } }");
    }
    #[test]
    pub fn t05_data_type_float() {
        let pre_string =
            "[T][table name][description]\n[Float][Float][Float]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[1.1e2][22.22][a]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to float. invalid float literal. row 0 col 2"
        );

        let data = "[-1.1e-2][-22.22][1,1]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to float. invalid float literal. row 0 col 2"
        );

        let data = "[-1.1e-2][1.1e2][0]\n";
        let s = format!("{}{}", pre_string, data);
        let table = unwrap!(Table::from_qvs20_str_with_schema(&s));
        assert_eq!(format!("{:?}", table), "Table { schema: TableSchema { table_name: \"table name\", table_description: \"description\", data_types: [Float, Float, Float], sub_table_schemas: [None, None, None], additional_properties: [\"prop1\", \"prop2\", \"prop3\"], column_names: [\"name1\", \"name2\", \"name3\"], active_row: 4, active_column: 3, row_delimiter: 10, sub_table_row_delimiter: 0 }, table_rows: TableRows { rows: [Row { values: [Float(-0.011), Float(110.0), Float(0.0)] }], table_name: \"\", active_row: 1, active_column: 0, row_delimiter: 10, sub_table_row_delimiter: 0 } }");
    }
    #[test]
    pub fn t06_data_type_bool() {
        let pre_string =
            "[T][table name][description]\n[Bool][Bool][Bool]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[T][F][a]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to bool. Only 84 and 70 allowed. Found: [97]. row 0 col 2"
        );

        let data = "[t][f][TF]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to bool. Only 84 and 70 allowed. Found: [116]. row 0 col 0"
        );

        let data = "[TFT][f][TF]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to bool. Only 84 and 70 allowed. Found: [84, 70, 84]. row 0 col 0"
        );
    }
    #[test]
    pub fn t07_data_type_datetime() {
        let pre_string = "[T][table name][description]\n[DateTimeFixedOffset][DateTimeFixedOffset][DateTimeFixedOffset]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[2014-11-28T21:00:09+09:00][2020-06-27T23:59:59+03:30][2014-11-28T21:00:09]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to datetime. premature end of input. row 0 col 2"
        );

        let data = "[2014-11-28T21:00:09+09:00][2020-06-27T23:59:59+03:30][2014-11-28]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to datetime. premature end of input. row 0 col 2"
        );

        let data =
            "[2014-11-28T21:00:09+09:00][2020-06-27T23:59:59+03:30][2014-11-28 21:00:09+09:00]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to datetime. input contains invalid characters. row 0 col 2"
        );

        let data = "[2014-11-28T21:00:09.12345678+09:00][2020-06-27T23:59:59+03:30][27.6.2020]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to datetime. input contains invalid characters. row 0 col 2"
        );
    }
    #[test]
    pub fn t08_data_type_date() {
        let pre_string =
            "[T][table name][description]\n[Date][Date][Date]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[2014-11-28][2020-06-27][2014-11-28+1]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to date. trailing input. row 0 col 2"
        );

        let data = "[2014-11-28][2020-06-27][27.6.2020]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to date. input contains invalid characters. row 0 col 2"
        );

        let data = "[2014-11-28][2020-06-27][2020/06/27]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to date. input contains invalid characters. row 0 col 2"
        );
    }
    #[test]
    pub fn t09_data_type_time() {
        let pre_string =
            "[T][table name][description]\n[Time][Time][Time]\n[][][]\n[prop1][prop2][prop3]\n[name1][name2][name3]\n";
        // rows of data active_row 0
        let data = "[23:59:59][23:59:59.123456][23:59]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to time. premature end of input. row 0 col 2"
        );

        let data = "[23:59:59][23:59:59.123456][99:99:99]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to time. input is out of range. row 0 col 2"
        );

        let data = "[23:59:59][23:59:59.123456][-23:59:00]\n";
        let s = format!("{}{}", pre_string, data);
        let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Failed conversion to time. input contains invalid characters. row 0 col 2"
        );
    }
    
    #[test]
    pub fn t03_write_schema_and_data() {
        let schema = TableSchema::new_simple_strings(3);
        let mut wrt = WriterForQvs20::new();
        schema.write_schema_to_writer(&mut wrt,false);
        wrt.write_string("three");
        wrt.write_string("o\\n[e]");
        wrt.write_string("t\nw\to\r");
        wrt.write_delimiter();
        let output = wrt.return_and_finish();
        assert_eq!(output, "[T][t1][simple table-only strings]\n[String][String][String]\n[][][]\n[][][]\n[1][2][3]\n[three][o\\\\n\\[e\\]][t\\nw\\to\\r]\n");
    }
}
