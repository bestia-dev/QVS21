// qvs20_table_rows_mod

//! The table structure is very flexible, because it is defined in runtime.
//! A lot of times the table is used just as an intermediary,
//! and don't need a fixed Rust struct in compile time.
//! It means that sometimes a change in the table does not dictate change in source code and compiling.

use crate::qvs20_reader_mod::*;
use crate::qvs20_table_schema_mod::*;
use crate::qvs20_writer_mod::*;
use crate::src_loc;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use rust_decimal::prelude::*;
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct TableRows {
    /// vector of rows. Can be empty.
    pub rows: Vec<Row>,

    // this is for error handling:
    pub table_name: String,
    /// index 0 for first row
    pub active_row: usize,
    /// index 0 for first column
    pub active_column: usize,
    /// row delimiter
    pub row_delimiter: u8,
    /// sub table row delimiter
    sub_table_row_delimiter: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub values: Vec<Value>,
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Decimal(Decimal),
    Float(f64),
    Bool(bool),
    DateTimeFixedOffset(DateTime<FixedOffset>),
    Date(NaiveDate),
    Time(NaiveTime),
    SubTable(TableRows),
}

impl Default for Value {
    fn default() -> Self {
        Value::String(s!())
    }
}

impl TableRows {
    pub fn new(table_name: &str, row_delimiter: u8) -> Result<TableRows, Qvs20Error> {
        let mut table_rows = TableRows::default();
        table_rows.table_name = s!(table_name);
        table_rows.row_delimiter = row_delimiter;
        // return
        Ok(table_rows)
    }

    /// rows from separate file than schema
    pub fn rows_from_qvs20_str(input: &str, schema: &TableSchema) -> Result<TableRows, Qvs20Error> {
        // the input is String to ensure it is well-formed utf8
        // internally use [u8] for performance
        let input = input.as_bytes();
        let mut rdr = ReaderForQvs20::new(input);
        let mut table_rows = TableRows::default();
        table_rows.row_delimiter = schema.row_delimiter;
        // first row is table_name and must be equal to schema
        table_rows.read_1st_row_file_type_and_table_name(&mut rdr)?;
        table_rows.append_data_rows(&mut rdr, schema)?;
        //return
        Ok(table_rows)
    }
    /// 1st row: file_type, table name, row_delimiter
    fn read_1st_row_file_type_and_table_name(
        &mut self,
        rdr: &mut ReaderForQvs20,
    ) -> Result<(), Qvs20Error> {
        let vec = rdr.next_row_as_vec_of_string()?;
        if vec.len() != 2 {
            return Err(Qvs20Error::Error {
                msg: format!("TableRows first row does not have 2 columns: file type and table name."),
            })
        }
        if vec[0] != "R"{
            return Err(Qvs20Error::Error {
                msg: format!("TableRows first field must be R for this file type"),
            })
        }
        self.table_name = vec[1].to_owned();
        // None is correct end of row
        // return
        Ok(())
    }

    /// internal append data to table after schema
    pub fn append_data_rows(
        &mut self,
        rdr: &mut ReaderForQvs20,
        schema: &TableSchema,
    ) -> Result<(), Qvs20Error> {
        while !rdr.peek_next_is_eof() && !rdr.peek_next_is_end_of_sub_table() {
            // data row
            while let Some(result) = self.while_append_one_data_row(rdr, schema) {
                // if Err then propagate
                result?;
            }
            self.active_row += 1;
        }
        //return
        Ok(())
    }

    /// data row
    /// Option::None means end of the row
    fn while_append_one_data_row(
        &mut self,
        rdr: &mut ReaderForQvs20,
        schema: &TableSchema,
    ) -> Option<Result<(), Qvs20Error>> {
        let result = match rdr.next() {
            Some(p) => p,
            None => {
                return Some(Err(Qvs20Error::Error {
                    msg: format!(
                        "Premature row end, col: {} {}",
                        self.active_column,
                        src_loc!()
                    ),
                }))
            }
        };
        let token = match result {
            Ok(p) => p,
            Err(e) => {
                return Some(Err(Qvs20Error::Error {
                    msg: format!("row {} {}{}", self.active_row, src_loc!(), err_trim!(e)),
                }))
            }
        };
        // create new row if needed
        if self.rows.len() <= self.active_row {
            self.rows.push(Row { values: vec![] });
        }
        match token {
            Token::Field(u) => {
                let value = match self.from_utf8_to_value(u, schema) {
                    Ok(v) => v,
                    Err(e) => {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!("{}{}", src_loc!(), err_trim!(e),),
                        }))
                    }
                };
                self.rows[self.active_row].values.push(value);
                self.active_column += 1;
                // return
                Some(Ok(()))
            }
            Token::RowDelimiter(r) => {
                // count of row columns must be same as in schema
                if self.active_column != schema.data_types.len() {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "row {} has {} columns differ from schema {} columns",
                            self.active_row,
                            self.active_column,
                            schema.data_types.len()
                        ),
                    }));
                }
                // row delimiter must be the same
                if r != self.row_delimiter {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "data row {} wrong row delimiter:{:?} instead of {:?} {}",
                            self.active_row,
                            r,
                            self.row_delimiter,
                            src_loc!()
                        ),
                    }));
                }
                self.active_column = 0;
                //end of row
                return None;
            }
            Token::StartSubTable(d) => {
                // start a new table
                let mut sub_table_rows = TableRows::default();
                sub_table_rows.row_delimiter = d;
                let sub_schema =
                    match &schema.sub_table_schemas[self.active_column] {
                        Some(s) => s,
                        None => {
                            return Some(Err(Qvs20Error::Error {
                                msg: format!(
                                "start sub table data row {} column {} missing sub table schema {}",
                                self.active_row, self.active_column, src_loc!()
                            ),
                            }))
                        }
                    };
                if sub_table_rows.row_delimiter != sub_schema.row_delimiter {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "start sub table row delimiter {} differs from sub schema {} {}",
                            sub_table_rows.row_delimiter,
                            sub_schema.row_delimiter,
                            src_loc!()
                        ),
                    }));
                }
                match sub_table_rows.append_data_rows(rdr, sub_schema) {
                    Ok(()) => (),
                    Err(e) => {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!("start sub table rows {}{}", src_loc!(), err_trim!(e),),
                        }))
                    }
                }
                self.rows[self.active_row]
                    .values
                    .push(Value::SubTable(sub_table_rows));
                // return
                Some(Ok(()))
            }
            Token::EndSubTable(d) => {
                let sub_schema =
                    match &schema.sub_table_schemas[self.active_column] {
                        Some(s) => s,
                        None => {
                            return Some(Err(Qvs20Error::Error {
                                msg: format!(
                                "end sub table data row {} column {} missing sub table schema {}",
                                self.active_row, self.active_column, src_loc!()
                            ),
                            }))
                        }
                    };
                if sub_schema.row_delimiter != d {
                    return Some(Err(Qvs20Error::Error {
                        msg: format!(
                            "End Sub Table Last delimiter is {} instead of {}. {}",
                            d,
                            self.row_delimiter,
                            src_loc!()
                        ),
                    }));
                };
                self.active_column += 1;
                // return
                Some(Ok(()))
            }
        }
    }
    /// for all types except sub_table
    fn from_utf8_to_value(&self, value: &[u8], schema: &TableSchema) -> Result<Value, Qvs20Error> {
        // various data types from utf8
        match schema.data_types[self.active_column] {
            DataType::String => match Self::from_u8_to_string(value) {
                Ok(s) => return Ok(Value::String(s)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: format!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Integer => match Self::from_u8_to_i64(value) {
                Ok(i) => return Ok(Value::Integer(i)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: format!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Decimal => match Self::from_u8_to_decimal(value) {
                Ok(d) => return Ok(Value::Decimal(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: format!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Float => match Self::from_u8_to_f64(value) {
                Ok(d) => return Ok(Value::Float(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: s!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Bool => match Self::from_u8_to_bool(value) {
                Ok(d) => return Ok(Value::Bool(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: s!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::DateTimeFixedOffset => match Self::from_u8_to_datetime(value) {
                Ok(d) => return Ok(Value::DateTimeFixedOffset(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: s!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Date => match Self::from_u8_to_date(value) {
                Ok(d) => return Ok(Value::Date(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: s!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            DataType::Time => match Self::from_u8_to_time(value) {
                Ok(d) => return Ok(Value::Time(d)),
                Err(e) => {
                    return Err(Qvs20Error::Error {
                        msg: s!(
                            "{}{}. row {} col {}",
                            src_loc!(),
                            err_trim!(e),
                            self.active_row,
                            self.active_column
                        ),
                    })
                }
            },
            // TODO: conversions
            DataType::SubTable => return Ok(Value::Integer(0)),
        };
    }

    pub fn from_u8_to_string(field_value: &[u8]) -> Result<String, Qvs20Error> {
        let str_value = match ReaderForQvs20::unescape(field_value) {
            Ok(s) => s,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: s!("Failed unescape to string. {}{}", src_loc!(), err_trim!(e)),
                })
            }
        };
        //return
        Ok(str_value)
    }

    pub fn from_u8_to_i64(field_value: &[u8]) -> Result<i64, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!(
                        "Failed conversion to string. {}{}",
                        src_loc!(),
                        err_trim!(e)
                    ),
                })
            }
        };
        let int_value = match str_value.parse::<i64>() {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!(
                        "Failed conversion to integer. {}{}",
                        src_loc!(),
                        err_trim!(e)
                    ),
                })
            }
        };
        //return
        Ok(int_value)
    }

    pub fn from_u8_to_decimal(field_value: &[u8]) -> Result<Decimal, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!(
                        "Failed conversion to string. {}{}",
                        src_loc!(),
                        err_trim!(e)
                    ),
                })
            }
        };
        let int_value = match Decimal::from_str(&str_value) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to decimal. {}", err_trim!(e),),
                })
            }
        };
        //return
        Ok(int_value)
    }
    pub fn from_u8_to_f64(field_value: &[u8]) -> Result<f64, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to string. {}", err_trim!(e)),
                })
            }
        };
        let f64_value = match str_value.parse::<f64>() {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to float. {}", err_trim!(e)),
                })
            }
        };
        //return
        Ok(f64_value)
    }
    pub fn from_u8_to_bool(field_value: &[u8]) -> Result<bool, Qvs20Error> {
        match field_value {
            &[b'T'] => return Ok(true),
            &[b'F'] => return Ok(false),
            _ => {
                return Err(Qvs20Error::Error {
                    msg: format!(
                        "Failed conversion to bool. Only {:?} and {:?} allowed. Found: {:?}",
                        b'T', b'F', field_value
                    ),
                })
            }
        };
    }
    /// rfc3339: 2014-11-28T21:00:09.123456+09:00
    pub fn from_u8_to_datetime(field_value: &[u8]) -> Result<DateTime<FixedOffset>, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to string. {}", err_trim!(e),),
                })
            }
        };
        let value = match DateTime::parse_from_rfc3339(&str_value) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to datetime. {}", err_trim!(e),),
                })
            }
        };
        //return
        Ok(value)
    }
    /// ISO 8601: 2014-11-28  no time zone. NaiveDate
    pub fn from_u8_to_date(field_value: &[u8]) -> Result<NaiveDate, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to string. {}", err_trim!(e),),
                })
            }
        };
        let value = match str_value.parse::<NaiveDate>() {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to date. {}", err_trim!(e),),
                })
            }
        };
        //return
        Ok(value)
    }
    /// ISO 8601: 23:59:59.123456  no time zone. NaiveTime
    pub fn from_u8_to_time(field_value: &[u8]) -> Result<NaiveTime, Qvs20Error> {
        let str_value = match String::from_utf8(field_value.to_vec()) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to string. {}", err_trim!(e),),
                })
            }
        };
        let value = match str_value.parse::<NaiveTime>() {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed conversion to time. {}", err_trim!(e),),
                })
            }
        };
        //return
        Ok(value)
    }
    /// write rows to String
    pub fn write_table_rows(&self) -> String {
        let mut wrt = WriterForQvs20::new();
        self.write_table_rows_to_writer(&mut wrt);
        //return
        wrt.return_and_finish()
    }
    /// write rows to writer
    pub fn write_table_rows_to_writer(&self, wrt: &mut WriterForQvs20) {
        if wrt.output_is_empty() {
            // when TableRows are in separate file from Schema
            // the 1st row has 2 fields: file type and TableName
            wrt.write_string("R");
            wrt.write_string(&self.table_name);
            wrt.write_delimiter();
        }
        for row in self.rows.iter() {
            for value in row.values.iter() {
                match value {
                    Value::String(s) => wrt.write_string(&s),
                    Value::Integer(i) => wrt.write_integer(*i),
                    Value::Decimal(d) => wrt.write_decimal(*d),
                    Value::Float(f) => wrt.write_float(*f),
                    /*
                    Bool(bool),
                    DateTimeFixedOffset(DateTime<FixedOffset>),
                    Date(NaiveDate),
                    Time(NaiveTime),
                    SubTable(TableRows),
                    */
                    _ => {}
                }
            }
            wrt.write_delimiter();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use unwrap::unwrap;

    #[test]
    pub fn t01_sub_table_separate_schema_and_rows() {
        let s = r"[S][table_name][description]
[String][SubTable][String]
[][1[U][vers][single col]1[String]1[]1[]1[ver]1][]
[blue][red][x]
[name][vers][desc]
";
        let schema = unwrap!(TableSchema::schema_from_qvs20_str(&s));
        let d = r"[R][table_name]
[name_1][1[1.0]1[1.1]1[2.1]1[2.2]1][desc_2]
[name_3][1[1.0]1[1.1]1[2.1]1[2.2]1][desc_4]
";
        let _rows = unwrap!(TableRows::rows_from_qvs20_str(&d, &schema));
    }
}
