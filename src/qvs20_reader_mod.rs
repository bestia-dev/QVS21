// qvs20_reader_mod

// normal fields are like
// [value]
// sub_table field have special syntax where one byte 1-9 are row_delimiters
// [1[one][two][many]1]
// [2[one][two][many]2]

use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::prelude::*;
use thiserror::Error;
use unwrap::unwrap;

#[derive(Error, Debug)]
pub enum Qvs20Error {
    // reader
    #[error("Error: {msg}")]
    Error { msg: String },
}

#[macro_export]
macro_rules! src_loc {
    () => {
        format!("{}:{}:{} ", file!(), line!(), column!())
    };
}
lazy_static! {
    static ref REGEX_SRC_LOC: Regex = unwrap!(Regex::new(r#"(src/.*? )"#));
}
/// src_loc is changing every time I edit the code
/// then all the error messages are different and the tests must be changed
/// I will remove src loc for error testing
pub fn remove_src_loc(err: Qvs20Error) -> String {
    let e = err.to_string();
    let after = REGEX_SRC_LOC.replace_all(&e, "");
    //return
    after.trim_end().to_string()
}

#[derive(Default)]
/// ReaderForQvs20
pub struct ReaderForQvs20<'a> {
    // All the fields are internal and not public.
    // The only way to interact is through methods.
    /// reference to the string (no allocation), but as [u8] for performance
    input: &'a [u8],
    /// where is the cursor now
    cursor_state: CursorState,
    /// cursor position
    cursor_pos: usize,

    // for better error handling
    // cursor pos is not good for error handling
    // for error handling I need row and column
    // in the format file_name.qvs20:6:8
    /// file_name for error handling
    file_name_for_error_handling: String,
    /// last new line cursor_pos
    last_new_line_cursor_pos: usize,
    /// row pos
    row: usize,
    /// row delimiter
    row_delimiter: u8,
}

/// internal enum
enum CursorState {
    /// start of file
    StartOfField,
    /// row_delimiter
    InsideRowDelimiter,
    /// inside of field  
    InsideOfField,
    /// outside of field
    OutsideOfField,
    /// reached normal end of file
    EndOfFile,
    /// Start sub_table
    StartSubTable,
    /// End sub_table
    EndSubTable,
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState::StartOfField
    }
}

/// the returned Token from the iterator
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token<'a> {
    /// field content - not unescaped  
    Field(&'a [u8]),
    /// row delimiter  
    RowDelimiter(u8),
    /// Start sub_table
    StartSubTable(u8),
    /// End sub_table
    EndSubTable(u8),
}

impl<'a> ReaderForQvs20<'a> {
    /// Constructor. String (but in [u8] form) is immutably borrowed here. No allocation.  
    pub fn new(input: &[u8]) -> ReaderForQvs20 {
        ReaderForQvs20 {
            input,
            file_name_for_error_handling: s!("./file.qvs20"),
            // default row_delimiter
            row_delimiter: b'\n',
            ..Default::default()
        }
    }
    pub fn set_row_delimiter(&mut self, row_delimiter: u8) {
        self.row_delimiter = row_delimiter;
    }
    /// errors msg have file_name:row:column position
    pub fn error_pos(&self) -> String {
        format!(
            "{}:{}:{}",
            self.file_name_for_error_handling,
            self.row,
            self.cursor_pos - self.last_new_line_cursor_pos
        )
    }
    /// low level - find u8 from pos_cursor
    pub fn find_u8_from(source_str: &[u8], pos_cursor: usize, find_u8: u8) -> Option<usize> {
        //print!("find_u8_from {}, {}, {}",unwrap!(String::from_utf8(source_str.to_vec())),pos_cursor,unwrap!(String::from_utf8(vec![find_u8])));
        let slice01 = &source_str[pos_cursor..];
        let opt_pos_found = slice01.iter().position(|&s| s == find_u8);
        if let Some(pos_found) = opt_pos_found {
            // return Option with usize
            //println!("return {} {}",pos_cursor,pos_found);
            Some(pos_cursor + pos_found)
        } else {
            //println!("return None");
            // return
            None
        }
    }
    /// peek if next character will be eof
    pub fn peek_next_is_eof(&self) -> bool {
        if self.cursor_pos + 1 >= self.input.len() {
            true
        } else {
            false
        }
    }
    /// peek if next character will be end of sub_table
    pub fn peek_next_is_end_of_sub_table(&self) -> bool {
        if self.input[self.cursor_pos + 1] == b']' {
            true
        } else {
            false
        }
    }
    /// peek if next character will be row_delimiter
    pub fn peek_next_is_row_delimiter(&self) -> bool {
        if self.input[self.cursor_pos] == self.row_delimiter {
            true
        } else {
            false
        }
    }

    /// unescape the qvs20 special 6 characters
    /// \\ Backslash character
    /// \[ Right Square Bracket
    /// \] Right Square Bracket
    /// \n New line
    /// \r Carriage return
    /// \t Tab
    pub fn unescape(field_value: &[u8]) -> Result<String, Qvs20Error> {
        //println!("unescape: {}", unwrap!(String::from_utf8(field_value.to_vec())));
        let mut ret = String::with_capacity(field_value.len());
        let mut start_pos = 0;
        while let Some(pos) = ReaderForQvs20::find_u8_from(field_value, start_pos, b'\\') {
            let end_pos = pos;
            // push the normal characters
            // cannot panic if the input was before well-formed utf8
            let str_value = unwrap!(String::from_utf8(field_value[start_pos..end_pos].to_vec()));
            ret.push_str(&str_value);
            // unescape the special character
            match field_value[end_pos + 1] {
                b'\\' => ret.push('\\'),
                b'[' => ret.push('['),
                b']' => ret.push(']'),
                b'n' => ret.push('\n'),
                b'r' => ret.push('\r'),
                b't' => ret.push('\t'),
                _ => {
                    return Err(Qvs20Error::Error {
                        msg: format!(
                            "Wrong escape sequence: \\{}",
                            unwrap!(String::from_utf8(vec![field_value[end_pos + 1]]))
                        ),
                    })
                }
            }
            start_pos = end_pos + 2;
        }
        // push the last part of normal characters
        let end_pos = field_value.len();
        // cannot panic if the input was before well-formed utf8
        let str_value = unwrap!(String::from_utf8(field_value[start_pos..end_pos].to_vec()));
        ret.push_str(&str_value);

        // return
        Ok(ret)
    }

    /// get next field as array of u8
    fn next_field_array_of_u8(&mut self) -> Result<&[u8], Qvs20Error> {
        let result = match self.next() {
            Some(p) => p,
            None => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected value found end of data {}", src_loc!(),),
                })
            }
        };
        let token = match result {
            Ok(p) => p,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("{}{}", src_loc!(), err_trim!(e)),
                })
            }
        };
        match token {
            Token::Field(field_value) => return Ok(field_value),
            _ => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected field not found {}", src_loc!(),),
                })
            }
        }
    }

    /// get next token and convert to string
    pub fn next_string(&mut self) -> Result<String, Qvs20Error> {
        let field_value = self.next_field_array_of_u8()?;
        match Self::unescape(field_value) {
            Ok(s) => return Ok(s),
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("Failed unescape to string. {}{}", src_loc!(), err_trim!(e)),
                })
            }
        }
    }
    /// get next token and convert to integer
    pub fn next_integer(&mut self) -> Result<i64, Qvs20Error> {
        let result = match self.next() {
            Some(p) => p,
            None => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected integer found end of data {}", src_loc!(),),
                })
            }
        };
        let token = match result {
            Ok(p) => p,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("{}{}", src_loc!(), err_trim!(e)),
                })
            }
        };
        match token {
            Token::Field(field_value) => {
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
            _ => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected field not found {}", src_loc!(),),
                })
            }
        }
    }

    /// get next token and convert to decimal
    pub fn next_decimal(&mut self) -> Result<Decimal, Qvs20Error> {
        let field_value = self.next_field_array_of_u8()?;
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
        let decimal_value = match Decimal::from_str(&str_value) {
            Ok(n) => n,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!(
                        "Failed conversion to decimal. {}{}",
                        src_loc!(),
                        err_trim!(e)
                    ),
                })
            }
        };
        //return
        Ok(decimal_value)
    }

    /// get next row_delimiter with check
    pub fn next_row_delimiter(&mut self) -> Result<(), Qvs20Error> {
        let result = match self.next() {
            Some(p) => p,
            None => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected row_delimiter found end of data {}", src_loc!(),),
                })
            }
        };
        let token = match result {
            Ok(p) => p,
            Err(e) => {
                return Err(Qvs20Error::Error {
                    msg: format!("{}{}", src_loc!(), err_trim!(e)),
                })
            }
        };
        match token {
            Token::RowDelimiter(r) => {
                if r != self.row_delimiter {
                    return Err(Qvs20Error::Error {
                        msg: format!(
                            "expected row_delimiter {}, but found {} {}",
                            self.row_delimiter,
                            r,
                            src_loc!(),
                        ),
                    });
                } else {
                    return Ok(());
                }
            }
            _ => {
                return Err(Qvs20Error::Error {
                    msg: format!("expected row_delimiter not found {}", src_loc!(),),
                })
            }
        }
    }
    /// next row as vec of string
    pub fn next_row_as_vec_of_string(&mut self) -> Result<Vec<String>, Qvs20Error> {
        let mut vec_of_string = vec![];
        while !self.peek_next_is_row_delimiter() {
            vec_of_string.push(unwrap!(self.next_string()));
        }
        unwrap!(self.next_row_delimiter());
        //return
        Ok(vec_of_string)
    }
}

impl<'a> Iterator for ReaderForQvs20<'a> {
    type Item = Result<Token<'a>, Qvs20Error>;
    /// Reads the next token. Return None when EndOfFile.
    /// Can return Token or Error.
    fn next(&mut self) -> Option<Result<Token<'a>, Qvs20Error>> {
        if self.input.is_empty() {
            return None;
        }
        // This loop breaks only with return
        loop {
            match &self.cursor_state {
                CursorState::StartOfField => {
                    if self.input[self.cursor_pos] == b'[' {
                        if self.cursor_pos + 2 >= self.input.len() {
                            return Some(Err(Qvs20Error::Error {
                                msg: format!(
                                    "Premature end of file. {} {}",
                                    self.error_pos(),
                                    src_loc!()
                                ),
                            }));
                        }
                        self.cursor_pos += 1;
                        // start of sub_table look like [1[ where 1-9 is the row delimiter and sub_table depth
                        if self.input[self.cursor_pos + 1] == b'['
                            && matches!(self.input[self.cursor_pos], b'1'..=b'9')
                        {
                            self.cursor_state = CursorState::StartSubTable;
                        } else {
                            self.cursor_state = CursorState::InsideOfField;
                        }
                    // continue loop
                    } else {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!(
                                "The field must start with [. {} {}",
                                self.error_pos(),
                                src_loc!()
                            ),
                        }));
                    }
                }
                CursorState::InsideOfField => {
                    let start_pos = self.cursor_pos;
                    while let Some(pos) = Self::find_u8_from(self.input, self.cursor_pos, b']') {
                        if self.input[pos - 1] == b'\\' {
                            // if before the delimiter is \ (escaped), then find the next
                            self.cursor_pos += 1;
                            if self.cursor_pos >= self.input.len() {
                                return Some(Err(Qvs20Error::Error {
                                    msg: format!(
                                        "Premature end of file. {} {}",
                                        self.error_pos(),
                                        src_loc!()
                                    ),
                                }));
                            }
                        //continue while
                        } else {
                            self.cursor_pos = pos;
                            break;
                        }
                    }
                    if self.input[self.cursor_pos] != b']' {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!(
                                "Last bracket is missing. {} {}",
                                self.error_pos(),
                                src_loc!()
                            ),
                        }));
                    }
                    let end_pos = self.cursor_pos;
                    self.cursor_pos += 1;
                    self.cursor_state = CursorState::OutsideOfField;
                    return Some(Ok(Token::Field(&self.input[start_pos..end_pos])));
                }
                CursorState::OutsideOfField => {
                    if self.cursor_pos >= self.input.len() {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!(
                                "Last row delimiter is missing. {} {}",
                                self.error_pos(),
                                src_loc!()
                            ),
                        }));
                    } else if self.input[self.cursor_pos] == b'[' {
                        self.cursor_state = CursorState::StartOfField;
                    } else {
                        self.cursor_state = CursorState::InsideRowDelimiter;
                    }
                }
                CursorState::InsideRowDelimiter => {
                    // row_delimiter must be only one byte \n, or 1-9 ascii
                    // the hierarchy will never be very deep. Probably till 3.
                    if self.cursor_pos + 1 >= self.input.len() {
                        self.cursor_state = CursorState::EndOfFile;
                        return Some(Ok(Token::RowDelimiter(self.input[self.cursor_pos])));
                    } else if self.input[self.cursor_pos + 1] == b']' {
                        self.cursor_state = CursorState::EndSubTable;
                        return Some(Ok(Token::RowDelimiter(self.input[self.cursor_pos])));
                    } else if self.input[self.cursor_pos + 1] == b'[' {
                        self.cursor_state = CursorState::StartOfField;
                        let start_pos = self.cursor_pos;
                        if self.input[self.cursor_pos] == b'\n' {
                            self.last_new_line_cursor_pos = self.cursor_pos;
                            self.row += 1;
                        }
                        self.cursor_pos += 1;
                        return Some(Ok(Token::RowDelimiter(self.input[start_pos])));
                    } else {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!(
                                "The row delimiter has more than 1 byte. {} {}",
                                self.error_pos(),
                                src_loc!()
                            ),
                        }));
                    }
                }
                CursorState::EndOfFile => {
                    // stop the iter() with None
                    return None;
                }
                CursorState::StartSubTable => {
                    // cursor is on the internal delimiter, all the checks are already made
                    let row_delimiter = self.input[self.cursor_pos];
                    self.cursor_pos += 1;
                    self.cursor_state = CursorState::StartOfField;
                    return Some(Ok(Token::StartSubTable(row_delimiter)));
                }
                CursorState::EndSubTable => {
                    // we are on the last delimiter of the sub_table
                    if self.cursor_pos + 2 >= self.input.len() {
                        return Some(Err(Qvs20Error::Error {
                            msg: format!(
                                "Premature end of sub_table. {} {}",
                                self.error_pos(),
                                src_loc!()
                            ),
                        }));
                    } else {
                        let row_delimiter = self.input[self.cursor_pos];
                        self.cursor_pos += 1;
                        self.cursor_pos += 1;
                        self.cursor_state = CursorState::OutsideOfField;
                        return Some(Ok(Token::EndSubTable(row_delimiter)));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use unwrap::unwrap;

    #[test]
    pub fn t00_remove_src_loc() {
        let s = Qvs20Error::Error {
            msg: s!("x-src/qvs20_reader_mod.rs:197:95 y"),
        };
        let after = remove_src_loc(s);
        assert_eq!(after, "Error: x-y");
    }

    #[test]
    pub fn t01_well_formed() {
        // fields, escaped, row_delimiter, eof
        let mut rdr = ReaderForQvs20::new(
            r"[one][two][1\\2\]3\[4\n5\r6\t]
[four]
"
            .as_bytes(),
        );
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field("one".as_bytes()));
        // second field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field("two".as_bytes()));
        // fourth field
        let token = unwrap!(unwrap!(rdr.next()));
        // here is raw bytes, not unescaped
        assert_eq!(token, Token::Field(r"1\\2\]3\[4\n5\r6\t".as_bytes()));
        // row_delimiter only one byte
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'\n'));
        // fifth field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(r"four".as_bytes()));
        // row_delimiter only one byte. Must end with row delimiter
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'\n'));
        // None is returned to signal the end for iter()
        let next = rdr.next();
        assert!(next.is_none());
    }

    #[test]
    pub fn t02_not_field() {
        let mut rdr = ReaderForQvs20::new("this is not a field".as_bytes());
        // first field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: The field must start with [. ./file.qvs20:0:0"
        );
    }

    #[test]
    pub fn t03_empty() {
        let mut rdr = ReaderForQvs20::new("".as_bytes());
        // first field
        let opt_result = rdr.next();
        assert!(opt_result.is_none());
    }

    #[test]
    pub fn t04_premature() {
        let mut rdr = ReaderForQvs20::new("[".as_bytes());
        // first field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: Premature end of file. ./file.qvs20:0:0"
        );
    }

    #[test]
    pub fn t05_no_last_bracket() {
        let mut rdr = ReaderForQvs20::new("[no last bracket".as_bytes());
        // first field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: Last bracket is missing. ./file.qvs20:0:1"
        );
    }

    #[test]
    pub fn t06_no_last_bracket() {
        let mut rdr = ReaderForQvs20::new("[one][no last bracket".as_bytes());
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field("one".as_bytes()));
        // second field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: Last bracket is missing. ./file.qvs20:0:6"
        );
    }
    #[test]
    pub fn t07_premature_end() {
        let mut rdr = ReaderForQvs20::new("[escaped \\]".as_bytes());
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: Premature end of file. ./file.qvs20:0:11"
        );
    }

    #[test]
    pub fn t08_no_row_delimiter() {
        let mut rdr = ReaderForQvs20::new("[one]".as_bytes());
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field("one".as_bytes()));
        // second field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: Last row delimiter is missing. ./file.qvs20:0:5"
        );
    }

    #[test]
    pub fn t09_row_delimiter_too_big() {
        let mut rdr = ReaderForQvs20::new("[one]\n\n".as_bytes());
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field("one".as_bytes()));
        // second field
        let result = unwrap!(rdr.next());
        let err = unwrap!(result.err());
        assert_eq!(
            remove_src_loc(err),
            "Error: The row delimiter has more than 1 byte. ./file.qvs20:0:5"
        );
    }
    #[test]
    pub fn t10_sub_table() {
        let mut rdr = ReaderForQvs20::new(
            "[one][1[sub1][sub2]1][three]\n[four][1[sub3][sub4]1[sub5][sub6]1][five]\n".as_bytes(),
        );
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"one"));
        // second field - sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::StartSubTable(b'1'));
        // sub_table first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub1"));
        // sub_table second field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub2"));
        // last delimiter of sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'1'));
        // end sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::EndSubTable(b'1'));
        // fourth field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"three"));
        // last delimiter of first row
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'\n'));

        // 2. row first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"four"));
        // second field - sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::StartSubTable(b'1'));
        // sub_table first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub3"));
        // sub_table second field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub4"));
        // last delimiter of sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'1'));
        // second row
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub5"));

        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub6"));
        // last delimiter of sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'1'));
        // end sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::EndSubTable(b'1'));
        // fourth field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"five"));
        // last delimiter of second row
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'\n'));
    }
    #[test]
    pub fn t11_sub_table_depth() {
        let mut rdr = ReaderForQvs20::new(
            "[one][1[sub1][2[s21][s22]2[s23][s24]2][sub3]1][three]\n".as_bytes(),
        );
        // first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"one"));
        // second field - sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::StartSubTable(b'1'));
        // sub_table first field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub1"));
        // second sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::StartSubTable(b'2'));

        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"s21"));
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"s22"));
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'2'));

        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"s23"));
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"s24"));
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'2'));
        // end sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::EndSubTable(b'2'));

        // sub_table second field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"sub3"));
        // last delimiter of sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'1'));
        // end sub_table
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::EndSubTable(b'1'));

        // fourth field
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::Field(b"three"));
        // last delimiter of first row
        let token = unwrap!(unwrap!(rdr.next()));
        assert_eq!(token, Token::RowDelimiter(b'\n'));
    }
    #[test]
    pub fn t12_unescape() {
        // one byte characters
        let unescaped = unwrap!(ReaderForQvs20::unescape(b"one"));
        assert_eq!(unescaped, "one");
        // normal processing of multi byte characters
        let unescaped = unwrap!(ReaderForQvs20::unescape("čćšß€".as_bytes()));
        assert_eq!(unescaped, "čćšß€");
        // qvs20 escape sequences
        let unescaped = unwrap!(ReaderForQvs20::unescape(r"1\]2\\3\r4\n5\t6\[".as_bytes()));
        assert_eq!(unescaped, "1]2\\3\r4\n5\t6[");
        // wrong escape sequence
        let err = ReaderForQvs20::unescape(r"12\34".as_bytes()).unwrap_err();
        assert_eq!(remove_src_loc(err), "Error: Wrong escape sequence: \\3");
        // wrong escape sequence for [ - the left bracket is NOT escaped, only the right bracket is escaped.
        let err = ReaderForQvs20::unescape(r"12\x34".as_bytes()).unwrap_err();
        assert_eq!(remove_src_loc(err), "Error: Wrong escape sequence: \\x");
    }
}
