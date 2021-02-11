// qvs20_table_schema_mod

//! The table structure is very flexible, because it is defined in runtime.
//! A lot of times the table is used just as an intermediary,
//! and don't need a fixed Rust struct in compile time.
//! It means that sometimes a change in the table does not dictate change in source code and compiling.

use crate::qvs20_reader_mod::*;
use crate::*;

use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Clone, Debug, Default)]
pub struct TableSchema {
    /// Schema 1st row - table name
    pub table_name: String,
    pub table_description: String,
    /// Schema 2nd row - data types
    pub data_types: Vec<DataType>,
    /// Schema 3rd row - SubTable schema
    pub sub_table_schemas: Vec<Option<TableSchema>>,
    /// Schema 4th row - additional properties as strings
    pub additional_properties: Vec<String>,
    /// Schema 5th row - column names, the last schema row
    pub column_names: Vec<String>,

    // this is for error handling
    /// index 0 for first row
    pub active_row: usize,
    /// active column
    pub active_column: usize,
    /// row_delimiter
    pub row_delimiter: u8,
    /// sub table row delimiter
    pub sub_table_row_delimiter: u8,
}

#[derive(Clone, Debug, EnumString, Display, Eq, PartialEq)]
pub enum DataType {
    String,
    Integer,
    Decimal,
    Float,
    Bool,
    DateTimeFixedOffset,
    Date,
    Time,
    SubTable,
}

// the read_field can return this variants for the schema fields
enum ValueForSchema {
    String(String),
    TableSchema(TableSchema),
}

impl TableSchema {
    /// a simple table with all Strings and automatic column_names
    pub fn new_simple_strings(count_of_column: usize) -> TableSchema {
        TableSchema {
            table_name: s!("t1"),
            table_description: s!("simple table-only strings"),
            row_delimiter: b'\n',
            data_types: vec![DataType::String; count_of_column],
            sub_table_schemas: vec![None; count_of_column],
            additional_properties: vec![s!(); count_of_column],
            column_names: (1..1 + count_of_column)
                .into_iter()
                .map(|x| s!(x))
                .collect(),
            active_row: 0,
            active_column: 0,
            sub_table_row_delimiter: 0,
        }
    }
    /// create a Schema object in memory from a qvs20 format string
    /// in the string is only the schema separated from the data
    pub fn schema_from_qvs20_str(input: &str) -> Result<TableSchema, Qvs20Error> {
        // the input is String to ensure it is well-formed utf8
        // internally use [u8] for performance
        let input = input.as_bytes();
        let mut rdr = ReaderForQvs20::new(input);

        let mut schema = TableSchema::default();
        schema.read_schema(&mut rdr)?;
        //return
        Ok(schema)
    }
    /// create a table schema in memory from a qvs20 format string
    pub fn read_schema(&mut self, rdr: &mut ReaderForQvs20) -> Result<(), Qvs20Error> {
        if self.row_delimiter == 0 {
            // default row delimiter if it is not already defined by the sub-table
            self.row_delimiter = b'\n';
        }

        self.read_row_schema_1st_row_table_name_and_description(rdr)?;
        self.read_row_schema_2nd_row_data_types(rdr)?;
        self.read_row_schema_3rd_row_sub_table_schemas(rdr)?;
        self.read_row_schema_4th_row_additional_properties(rdr)?;
        self.read_row_schema_5th_row_column_names(rdr)?;

        //return
        Ok(())
    }
    fn active_row_str(&self) -> String {
        // humans count from 1, machines count from 0
        let a = match self.active_row {
            0 => "1st row - file_type, table name and description",
            1 => "2nd row - data types",
            2 => "3rd row - SubTable schemas",
            3 => "4th row - additional properties",
            4 => "5th row - column names",
            _ => "",
        };
        //return
        s!(a)
    }

    /// Schema 1st row: table name, description and row_delimiter are mandatory
    fn read_row_schema_1st_row_table_name_and_description(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        // there are 3 possibilities: 1. correct token, 2. correct end of row, 3. a bunch of errors
        self.active_row = 0;
        self.active_column = 0;
        while let Some(result) = self.read_field_in_row(rdr) {
            // correct token, or propagate error
            match result? {
                ValueForSchema::String(value) => {
                    if self.active_column == 0 {
                        let _file_type = value;
                    } else if self.active_column == 1 {
                        self.table_name = value;
                    } else if self.active_column == 2 {
                        self.table_description = value;
                    }
                    self.active_column += 1;
                }
                ValueForSchema::TableSchema(_) => {
                    return Err(Qvs20Error::Error {
                        msg: format!("Schema {}", self.active_row_str()),
                    })
                }
            }
        }
        // None is correct end of row
        // return
        Ok(())
    }

    /// Schema 2nd row: data types
    fn read_row_schema_2nd_row_data_types(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        // there are 3 possibilities: 1. correct token, 2. correct end of row, 3. a bunch of errors
        self.active_row = 1;
        self.active_column = 0;
        while let Some(result) = self.read_field_in_row(rdr) {
            // correct token, or propagate error
            match result? {
                ValueForSchema::String(data_type) => {
                    let data_type = match DataType::from_str(&data_type) {
                        Ok(p) => p,
                        Err(e) => {
                            return Err(Qvs20Error::Error {
                                msg: format!("Schema 2nd row {}", err_trim!(e)),
                            })
                        }
                    };
                    self.data_types.push(data_type);
                    self.active_column += 1;
                }
                ValueForSchema::TableSchema(_) => {
                    return Err(Qvs20Error::Error {
                        msg: format!("Schema {}", self.active_row_str()),
                    })
                }
            }
        }
        // None is correct end of row
        // return
        Ok(())
    }

    /// Schema 3rd row - sub tables schema
    fn read_row_schema_3rd_row_sub_table_schemas(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        // there are 3 possibilities: 1. correct token, 2. correct end of row, 3. a bunch of errors
        self.active_row = 2;
        self.active_column = 0;
        while let Some(result) = self.read_field_in_row(rdr) {
            // correct token, or propagate error
            match result? {
                ValueForSchema::String(s) => {
                    if s.is_empty() {
                        self.sub_table_schemas.push(None);
                    } else {
                        return Err(Qvs20Error::Error {
                            msg: format!(
                                "Schema {} Expected TableSchema found {}.",
                                self.active_row_str(),
                                s
                            ),
                        });
                    }
                }
                ValueForSchema::TableSchema(schema) => self.sub_table_schemas.push(Some(schema)),
            }
            self.active_column += 1;
        }
        // None is correct end of row
        // return
        Ok(())
    }

    /// Schema 4th row - additional properties
    fn read_row_schema_4th_row_additional_properties(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        // there are 3 possibilities: 1. correct token, 2. correct end of row, 3. a bunch of errors
        self.active_row = 3;
        self.active_column = 0;
        while let Some(result) = self.read_field_in_row(rdr) {
            // correct token, or propagate error
            match result? {
                ValueForSchema::String(property) => {
                    self.additional_properties.push(property);
                    self.active_column += 1;
                }
                ValueForSchema::TableSchema(_) => {
                    return Err(Qvs20Error::Error {
                        msg: format!("Schema {}", self.active_row_str()),
                    })
                }
            }
        }
        // None is correct end of row
        // return
        Ok(())
    }

    /// Schema 5th row - column names
    fn read_row_schema_5th_row_column_names(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        // there are 3 possibilities: 1. correct token, 2. correct end of row, 3. a bunch of errors
        self.active_column = 0;
        self.active_row = 4;
        while let Some(result) = self.read_field_in_row(rdr) {
            // correct token, or propagate error
            match result? {
                ValueForSchema::String(column_name) => {
                    // names of columns must be unique
                    for name in self.column_names.iter() {
                        if name == &column_name {
                            return Err(Qvs20Error::Error {
                                msg: format!("Column names must be unique: {}", &column_name),
                            });
                        }
                    }
                    self.column_names.push(column_name);
                    self.active_column += 1;
                }
                ValueForSchema::TableSchema(_) => {
                    return Err(Qvs20Error::Error {
                        msg: format!("Schema {}", self.active_row_str()),
                    })
                }
            }
        }
        // None is correct end of row
        // return
        Ok(())
    }

    /// read field in row, returns ValueForSchema: variants String or SubTable Schema
    /// None means end of the row
    fn read_field_in_row(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Option<Result<ValueForSchema, Qvs20Error>> {
        let result = match rdr.next() {
            Some(p) => p,
            None => {
                return Some(Err(Qvs20Error::Error {
                    msg: format!("Missing mandatory Schema {}.", self.active_row_str()),
                }))
            }
        };
        let token = match result {
            Ok(p) => p,
            Err(e) => {
                return Some(Err(Qvs20Error::Error {
                    msg: format!("Schema {} {}", self.active_row_str(), err_trim!(e)),
                }))
            }
        };
        match token {
            Token::Field(value) => {
                let value = match ReaderForQvs20::unescape(value) {
                    Ok(p) => p,
                    Err(e) => {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!("Schema {}: {}", self.active_row_str(), err_trim!(e)),
                        }))
                    }
                };
                return Some(Ok(ValueForSchema::String(value)));
            }
            Token::RowDelimiter(r) => {
                // 1st row must have 3 columns
                if self.active_row == 0 && self.active_column != 3 {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!("Schema {} must have 3 columns.", self.active_row_str()),
                    }));
                }
                // all other rows must have same column count as the 2nd row - data types
                if self.active_row != 0 && self.active_column != self.data_types.len() {
                    return Some(Err(Qvs20Error::Error{
                        msg:format!("Schema {} must have same number of columns as Schema 2nd row data types.",self.active_row_str()),
                    }));
                }
                // row delimiter must be the same
                if r != self.row_delimiter {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "Schema {} wrong row delimiter:{:?} instead of {:?}",
                            self.active_row_str(),
                            r,
                            self.row_delimiter
                        ),
                    }));
                }
                //end of row
                return None;
            }
            Token::StartSubTable(d) => {
                self.sub_table_row_delimiter = d;
                // start a new sub schema
                let mut schema = TableSchema::default();
                schema.row_delimiter = d;
                match schema.read_schema(rdr) {
                    Ok(()) => (),
                    Err(e) => {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!("sub table schema {}", err_trim!(e)),
                        }))
                    }
                }
                // recursive call for mandatory next token EndSubTable
                // this 2 tokens are like one, they must be in pair.
                match self.read_field_in_row(rdr) {
                    None => {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!("Schema {} EndSubTable missing.", self.active_row_str()),
                        }))
                    }
                    Some(result) => match result {
                        Err(e) => {
                            return Some(Err(Qvs20Error::Error {
                                msg: format!("EndSubTable {}", err_trim!(e)),
                            }))
                        }
                        Ok(value) => match value {
                            // finally the happy return
                            ValueForSchema::String(s) => {
                                if s.is_empty() {
                                    return Some(Ok(ValueForSchema::TableSchema(schema)));
                                } else {
                                    return Some(Err(Qvs20Error::Error {
                                        msg: format!(
                                            "Schema {} EndSubTable",
                                            self.active_row_str()
                                        ),
                                    }));
                                }
                            }
                            ValueForSchema::TableSchema(_) => {
                                return Some(Err(Qvs20Error::Error {
                                    msg: format!("Schema {} EndSubTable", self.active_row_str()),
                                }))
                            }
                        },
                    },
                }
            }
            Token::EndSubTable(d) => {
                if self.sub_table_row_delimiter == 0 {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "Schema {} EndSubTable missing schema in sub_table_schema.",
                            self.active_row_str()
                        ),
                    }));
                } else if self.sub_table_row_delimiter != d {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "Schema {} EndSubTable Last delimiter is {} instead of {}.",
                            self.active_row_str(),
                            d,
                            self.row_delimiter
                        ),
                    }));
                } else {
                    self.sub_table_row_delimiter = 0;
                    // this is called from recursive call in Token::StartSubTable
                    // so I can return:
                    return Some(Ok(ValueForSchema::String(s!())));
                }
            }
        }
    }

    // write standalone schema to String
    pub fn write_schema(&self) -> String {
        let mut wrt = WriterForQvs20::new();
        self.write_schema_to_writer(&mut wrt, true);
        //return
        wrt.return_and_finish()
    }

    /// write to writer
    pub fn write_schema_to_writer(&self, wrt: &mut WriterForQvs20, schema_only:bool) {
        if schema_only==true{
            wrt.write_string("S");
        }else{
            wrt.write_string("T");
        }
        wrt.write_string(&self.table_name);
        wrt.write_string(&self.table_description);
        wrt.write_delimiter();
        for x in self.data_types.iter() {
            wrt.write_string(&x.to_string());
        }
        wrt.write_delimiter();
        for x in self.sub_table_schemas.iter() {
            match x {
                None => wrt.write_string(""),
                Some(schema) => wrt.write_sub_table_schema(&schema),
            }
        }
        wrt.write_delimiter();
        for x in self.additional_properties.iter() {
            wrt.write_string(&x);
        }
        wrt.write_delimiter();
        for x in self.column_names.iter() {
            wrt.write_string(&x);
        }
        wrt.write_delimiter();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    //use unwrap::unwrap;

    #[test]
    pub fn t01_all_wrong_in_schema() {
        // error in Schema 1st row - table_name
        let s = r"";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Missing mandatory Schema 1st row - file_type, table name and description."
        );

        let s = r"bad-formed text";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 1st row - file_type, table name and description The field must start with [. ./file.qvs20:0:0"
        );

        let s = r"[bad-formed text";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 1st row - file_type, table name and description Last bracket is missing. ./file.qvs20:0:1"
        );

        let s = r"[S]";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 1st row - file_type, table name and description Last row delimiter is missing. ./file.qvs20:0:3"
        );

        let s = r"[S][name][desc]";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 1st row - file_type, table name and description Last row delimiter is missing. ./file.qvs20:0:15"
        );

        let s = "[S][name][row delimiter too big]\n\n";
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 1st row - file_type, table name and description The row delimiter has more than 1 byte. ./file.qvs20:0:32"
        );

        // error in Schema 2nd row - data types
        // good table_name, but no 2nd row
        let pre_string = "[S][table name][description]\n";
        let s = format!("{}{}", pre_string, "");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Missing mandatory Schema 2nd row - data types."
        );

        let s = format!("{}{}", pre_string, "[String");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 2nd row - data types Last bracket is missing. ./file.qvs20:1:2"
        );

        let s = format!("{}{}", pre_string, "[String]");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 2nd row - data types Last row delimiter is missing. ./file.qvs20:1:9"
        );

        let s = format!("{}{}", pre_string, "[String][Integer][Decimal]");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 2nd row - data types Last row delimiter is missing. ./file.qvs20:1:27"
        );

        // wrong row delimiter
        let s = format!("{}{}", pre_string, "[String][Integer][Decimal]1");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 2nd row - data types wrong row delimiter:49 instead of 10"
        );

        let s = format!("{}{}", pre_string, "[String][Integer][Decimal]\n\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 2nd row - data types The row delimiter has more than 1 byte. ./file.qvs20:1:27"
        );
        // for Schema 3rd row -sub table schema there is separate test

        // error in Schema 4th row - additional properties
        let pre_string = "[S][table name][description]\n[String][Integer][Decimal]\n[][][]\n";
        let s = format!("{}{}", pre_string, "");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Missing mandatory Schema 4th row - additional properties."
        );
        let s = format!("{}{}", pre_string, "[prop");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 4th row - additional properties Last bracket is missing. ./file.qvs20:3:2"
        );

        let s = format!("{}{}", pre_string, "[prop]");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 4th row - additional properties Last row delimiter is missing. ./file.qvs20:3:7"
        );

        let s = format!("{}{}", pre_string, "[prop][prop]\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 4th row - additional properties must have same number of columns as Schema 2nd row data types."
        );

        // wrong row delimiter
        let s = format!("{}{}", pre_string, "[prop1][prop2][prop3]1");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 4th row - additional properties wrong row delimiter:49 instead of 10"
        );

        // row delimiter must be only 1 byte
        let s = format!("{}{}", pre_string, "[prop1][prop2][prop3]\n\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 4th row - additional properties The row delimiter has more than 1 byte. ./file.qvs20:3:22"
        );

        // Schema 5th row: column names
        let pre_string = "[S][table name][description]\n[String][Integer][Decimal]\n[][][]\n[prop1][prop2][prop3]\n";
        let s = format!("{}{}", pre_string, "");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Missing mandatory Schema 5th row - column names."
        );

        let s = format!("{}{}", pre_string, "[name");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 5th row - column names Last bracket is missing. ./file.qvs20:4:2"
        );

        let s = format!("{}{}", pre_string, "[name]");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 5th row - column names Last row delimiter is missing. ./file.qvs20:4:7"
        );

        let s = format!("{}{}", pre_string, "[name1][name2]\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(remove_src_loc(err), "Error: Schema 5th row - column names must have same number of columns as Schema 2nd row data types.");

        // column names must be unique
        let s = format!("{}{}", pre_string, "[name1][name1][name1]\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Column names must be unique: name1"
        );

        // wrong row delimiter
        let s = format!("{}{}", pre_string, "[name1][name2][name3]1");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 5th row - column names wrong row delimiter:49 instead of 10"
        );

        let s = format!("{}{}", pre_string, "[name1][name2][name3]\n\n");
        let err = TableSchema::schema_from_qvs20_str(&s).unwrap_err();
        assert_eq!(
            remove_src_loc(err),
            "Error: Schema 5th row - column names The row delimiter has more than 1 byte. ./file.qvs20:4:22"
        );
    }
    #[test]
    pub fn t02_write_schema() {
        let schema = TableSchema::new_simple_strings(3);
        let mut wrt = WriterForQvs20::new();
        schema.write_schema_to_writer(&mut wrt,true);
        let output = wrt.return_and_finish();
        assert_eq!(
            output,
            "[S][t1][simple table-only strings]\n[String][String][String]\n[][][]\n[][][]\n[1][2][3]\n"
        );
    }
}
