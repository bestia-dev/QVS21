// region: lmake_md_to_doc_comments include README.md A //!
//! # QVS20 - Modern replacement for csv for the year 2020  
//!
//! sQuare brackets Separated Values 2020  
//!
//! ***version: 0.0.8  date: 2020-07-16 authors: bestia.dev***  
//! **Modern replacement for csv for the year 2020**
//!
//! [![Lines in Rust code](https://img.shields.io/badge/Lines_in_Rust-1574-green.svg)](https://github.com/bestia-dev/QVS20/)
//! [![Lines in Doc comments](https://img.shields.io/badge/Lines_in_Doc_comments-612-blue.svg)](https://github.com/bestia-dev/QVS20/)
//! [![Lines in Comments](https://img.shields.io/badge/Lines_in_comments-211-purple.svg)](https://github.com/bestia-dev/QVS20/)
//! [![Lines in examples](https://img.shields.io/badge/Lines_in_examples-954-yellow.svg)](https://github.com/bestia-dev/QVS20/)
//! [![Lines in tests](https://img.shields.io/badge/Lines_in_tests-812-orange.svg)](https://github.com/bestia-dev/QVS20/)
//!
//!   
//! My proposed format for import/export of 2 dimensional database tables.  
//!   
//! ## The name QVS20  
//!   
//! `QVS` stands for "sQuare brackets Separated Values".  
//! The number `20` is the year of the first version: 2020.  
//! Read phonetically as:  
//!   
//! - International Phonetic Alphabet (IPA) [kʊs twɛnti]
//! - Americanist Phonetic Notation [koos twen-tee]
//!   
//! Read pronunciation like `couscous` the North African dish or `Kuss` the German or Dutch words for Kiss.  
//! <https://en.wiktionary.org/wiki/kus>  
//! A minor annoyance is that in Crimean Tatar kuş means bird or penis and in Kurdish kuş means fart.  
//! But stay positive I say :-)  
//!
//! ## Check the development of the library source code
//!
//! Clone and try the examples.  
//! The description is in the code and in <DEVELOPMENTS.md>.  
//!
//! ## Text format for tabular data  
//!   
//! Most databases are used to store information in tables - the tabular form of 2 dimensional data structure.  
//! The structure is very simple to understand and visualize.  
//! This data quite often need to be exported, transferred and imported between very different systems.  
//! Binary transfer is hard to implement in heterogeneous systems.  
//! Text transfer is slower and larger, but is universal for all computer systems and even human readable. The internet has good experience of that.  
//! The text format is probably very good to compress/zip if needed.  
//!
//! ## Machines first - Humans second
//!
//! `QVS20` is primarily intended to be write and read by computer code.  
//! It is still possible to read and edit it manually.  
//! All in all it is just a simple text.  
//! But that is not very comfortable and it is absolutely not the primary goal.  
//! So this is not a good format for config files.  
//! Just for your info, no comments are allowed in the standard.  
//! Only data.  
//!   
//! ## Database table  
//!   
//! A database table is a structure that organizes data into rows and columns - forming a grid.  
//! The rows run horizontally and represent each record.  
//! The columns run vertically and represent a specific field.  
//! A table has a specified number of columns.  
//! Every column has a description what kind of data it represents.  
//! The `QVS20` format can be used also for hierarchical data structures, but it does not shine there.  
//! We already have `xml` and `json` for that type of data.  
//! Today we need a format for (mostly) tabular data.  
//!
//! ## A modern replacement for csv  
//!   
//! `csv` is the most used format to export/import tables.  
//! But it is not even a official standard. Even the name is deceiving: CSV comma separated values. Sometimes the delimiter is not "comma", but tab, semicolon,...  
//! Every programmer has a different implementation.  
//! There are a million small problems that makes its use problematic for different systems.  
//! Though it works fine for import/export in the same system where the programmers made the same decisions.  
//! Csv has the smallest footprint for a text format, but :  
//!   
//! - what is the field delimiter: comma, semi-colon, colon, tab, ?  
//! - what is the row delimiter: newline, CRLF, CR, LF?  
//! - how to escape the delimiters inside the content?  
//! - if we use the quotes as delimiters, how to escape the quotes?  
//! - do we use the quotes everywhere or only on selected fields?  
//! - what encoding is used for the file?  
//! - how the decimal number are represented?  
//! - how the date/time is represented?  
//! - there is no standard for field definition.  
//! - The data could come without column names.  
//!
//! example of csv:  
//!
//! ```txt
//! Date Received,Product Name,Sub Product,Issue,Sub Issue,Consumer Complaint Narrative,Company Public Response,Company,State Name,Zip Code,Tags,Consumer Consent Provided,Submitted via,Date Sent to Company,Company Response to Consumer,Timely Response,Consumer Disputed,Complaint ID
//! 2013-07-29,Consumer Loan,Vehicle loan,Managing the loan or lease,,,,Wells Fargo & Company,VA,24540,,N/A,Phone,2013-07-30,Closed with explanation,Yes,No,468882
//! 2013-07-29,Bank \ account or service,Checking account,Using a debit or ATM card,,,,Wells Fargo & Company,CA,95992,Older American,N/A,Web,2013-07-31,Closed with explanation,Yes,No,468889
//!   ```
//!
//! `xml` and `json` are great inter-system formats for hierarchically structured data, but not for tabular data.  
//! They are terribly verbose for tables, because they repeat all the names for every row of data.  
//! Imagine a table with a million rows. Most of this data is field names. Horror!  
//!
//! example of json:  
//!
//! ```json
//! [{
//!         "country": "Afghanistan",
//!         "population": "37172386"
//!     },
//!     {
//!         "country": "Albania",
//!         "population": "2866376"
//!     },
//!     {...
//! ```
//!
//! example of xml with elements:
//!
//! ```xml
//!     <row>
//!         <country>Afghanistan</country>
//!         <population>37172386</population>
//!     </row>
//!     <row>
//!     <country>Albania</country>
//!         <population>2866376</population>
//!     </row>
//!     <row>...
//! ```
//!
//! example of xml with attributes:
//!
//! ```xml with attributes
//!     <row country="Afghanistan" population="37172386" />
//!     <row country="Albania" population="2866376" />
//!     <row ...
//! ```
//!
//! There is probably a lot more formats out there, but I cannot know them all.  
//!   
//! ## Why sQuare brackets  
//!   
//! I think parsing is much easier and error resistent if the delimiters are different for start and for end.  
//! It is the simplest rule that all the delimiters must always exist. No exceptions.  
//! If we compare to `csv` it adds one additional delimiter, but that solves a lot of problems.  
//! Why not quotes or apostrophes : they are the same delimiter for start and end.  
//! Why not <> like xml? : If we have a lot of xml data, then all of these delimiters must be escaped.  
//! Why not {}? : if we have a lot of json or code, there is a lot of characters to be escaped.  
//! `sQuare brackets` [] are used in many places, but less then other symbols.  
//! They are ascii7, just one byte in UTF8 and easy/fast to parse.  
//! example of QVS20:  
//!
//! ```QVS20
//! [City][Country][UN2018][CityPopulation][Area][MetropolitanPopulation]  
//! [Tokyo][Japan][37400068][13515271][2191][37274000]  
//! [Delhi][India][28514000][16753235][1484][29000000]  
//! ```
//!
//! ## Only utf-8 Encoding
//!   
//! There is a few different text encodings today. All come with good and bad properties.  
//! In 2020 the choice to simplify most of the cases is `utf-8`.  
//! It is much easier to maintain the source code for `QVS20` import/export for only that one encoding.  
//! The utf-8 encoding is very efficient most of the times.  
//! The Rust programming language uses utf-8 internally for the String type.  
//! Utf-8 is quite complex.  
//! There can be characters with 1, 2, 3, 4, 5, 6 bytes.  
//! But the mastery of the encoding is that it `guarantees` that the same byte (number) for an ascii7 characters can never never be found anywhere else in any other character. It does not matter how much complicated is the 6 byte character. No one of that bytes can have a number less than 127.  
//! It is a guarantee! Genius !  
//! That means that we can approach an utf-8 file as a vector of bytes. Super simple, fast and easy. We can just ignore all the peculiarities of millions of unicode characters. Because we care only about our delimiters.  
//! And they are all 100% ascii7.  
//!   
//! ## Visualization  
//!   
//! This format is mostly made for machines. There is not superfluous data for visualization like spaces or tabs or newlines or comments.  
//! It is still human readable, but not very comfortable without special visualizers/editors.  
//! If your editor is clever and knows how to add a few spaces here and there, this format can become easy human readable.  
//! This could be easily processed by all new editor/visualizer with extensions.  
//! But remember, this is not the truth in the file.  
//! We want small file sizes and there are no spaces in the file.  
//! One problem of visualizations are long strings. They are hard to place on the screen in a tabular format to be nicely readable.  
//! One idea to solve this "problem" is to add in the schema an optional field property `MaxVisualizeLength`. The visualizer should truncate the long string to achieve great readability. In most of the cases, we don't want to see the whole long long strings every time. Mostly the info in long long string is not really important.  
//!   
//! Example of visualization:  
//!
//! ```QVS20  
//! [City]  [Country] [UN2018]   [CityPopulation] [Area] [MetropolitanPopulation]  
//! [Tokyo] [Japan]   [37400068] [13515271]       [2191] [37274000]  
//! [Delhi] [India]   [28514000] [16753235]       [1484] [29000000]  
//! ```
//!
//! ## Common data types  
//!   
//! A standard must be prepared for common data types and not leave them to different interpretations, because that can cause incompatibilities.  
//! This data is mostly used in database tables, so the datatypes must be understandable in the database world. But the import/export utilities are write in languages that have different data types.  
//! Here we need something in between that is understandable to both.  
//! Here are the most used data types.  
//! For exotic datatypes there is no need to define them in this standard.  
//! Maybe in some next standard if that data type becomes really important.  
//!   
//! ## Data type String
//!   
//! Utf-8 enables to represent any unicode character.  
//! We just need to be aware of this 4 special ascii7 characters:  
//!   
//! - [ left square brackets  
//! - ] right square brackets  
//! - LF new line  
//! - \  Backslash character  
//!   
//! We can use the backslash character to escape the special characters:  
//!
//! - \\\\  Backslash character  
//! - \\[  Left Square Bracket  
//! - \\]  Right Square Bracket  
//! - \\n  New line  
//!   
//! We want to avoid undesirable editing characters in long strings, so we escape also these 2:  
//!
//! - \r  Carriage return  
//! - \t  Tab  
//!
//! So all together there are 6 ascii7 characters of interest to the parser and serializer.
//!   
//! ## Data type Integer
//!   
//! Allowed characters: numbers and eventually a minus/plus before them.  
//! [5]  
//! [+5]  
//! [-6]  
//!   
//! ## Data type Decimal
//!   
//! The decimal type is very important in databases and needs to be explicit.  
//! The decimal is NOT the same as the float, because the float cannot contain an exact value ! Float makes it super problematic for any financial calculation or number comparison.  Use the Decimal type instead.  
//! Allowed characters: numbers and decimal point and the minus/plus characters.  
//! No space, no commas, no thousand delimiters, no currency symbols.  
//! No binary, hexadecimal or other numbers are allowed here.  
//!
//! [3]  
//! [+3]  
//! [-2]  
//! [4.1]  
//! [+4.1]  
//! [-5.0]  
//! [9.23872000]  
//!   
//! ## Data type Float
//!   
//! Avoid floats in "real life" scenarios. Use them only for scientific calculations. A classic problem with floats (inaccurate): `1.2 - 1.0 = 0.199999999999999996` ?!? This comparison `0.2 == (1.2 - 1.0)` returns `false` !?!  
//! Allowed characters: numbers and decimal point and the minus/plus characters and the `e` exponent.  
//! [2.99792458e8]  
//! [+2.99792458e8]  
//! [-2.99792458e8]  
//! [2.99792458e-8]  
//!   
//! ## Type Bool  
//!   
//! `F` for false  
//! `T` for true  
//! It is case sensitive, uppercase only.  
//! This characters are ascii7 and in utf-8 they use one byte only.  
//!   
//! [T]  
//! [F]  
//!
//! ## Data type Date
//!   
//! Only one format: `YYYY-MM-DD`  
//! All the parts are required.  
//! No TimeZone or Fixed Offset.  
//! [2002-09-24]  
//!   
//! ## Data type Time
//!   
//! Only one format: `hh:mm:ss.dddd`  
//! `.dddd` is optional. All the rest is required.  
//! No TimeZone or Fixed Offset.  
//! [23-59-59]  
//! [23-59-59.12345]  
//!   
//! ## Data type DateTime with FixedOffset
//!   
//! Only one format: `YYYY-MM-DDThh:mm:ss.dddd+hh:mm`  
//! The FixedOffset is mandatory.  
//! `.dddd` is optional. All the rest is required.  
//! [2002-05-30T09:30:10.5]  
//!   
//! ## Data type binary  
//!   
//! Binary data should be converted into Base64 and used as very long string.  
//! This is not the best way to transfer big binary data.  
//! The Base64 does not include any of the 4 special characters.  
//! So the parser cannot be confused.  
//!
//! ## Null  
//!   
//! Null in database language means `absence of data` and is very important.  
//! Empty brackets [] represent Null in `QVS20`.  
//! For strings there is no difference between Null and EmptyString in `QVS20`.  
//!
//! ## Data type SubTable
//!   
//! Inside one cell of the table is possible to insert a whole sub-table.  
//! Just like that. Nothing changes, no special escaping, because we have start and end delimiters.  
//! So we can represent also hierarchical data, if it is really needed.  
//! Still the primary goal of the standard is tabular data.  
//! Yes, but one tiny thing must change: the row delimiter of the sub-table is not \n anymore.  
//! It is the number that represent the depth of the sub-table: 1, 2, 3,...  
//! It must be only one byte. This format is not really great for hierarchies deeper than 9 levels.  
//!
//! Example of CountryTable:  
//!
//! ```QVS20
//! [Country][Population]  
//! [Slovenia][2000000]  
//! [Italia][60000000]  
//! [Croatia][4000000]  
//! ```
//!   
//! Now we want to insert the data of the cities, but hierarchically as a sub-table.  
//! Example of CityDataTable for Slovenia:  
//!
//! ```QVS20
//! [City][MetropolitanPopulation]  
//! [Ljubljana][300000]  
//! [Koper][30000]  
//! ```
//!
//! Together table and sub-table looks like this:  
//! The sub-table row delimiter is changed to "1".  
//! For easy parsing, the sub-table starts with the new row delimiter.  
//!   
//! ```QVS20
//! [Country][CityDataSubTable][Population]  
//! [Slovenia][1[Ljubljana][300000]1[Koper][30000]1][2000000]  
//! [Italia][1[Milano][400000]1[Venezia][30000]1][60000]  
//! ```
//!
//! This is not very human readable, because the lack of spaces.  
//! But you can visualize it like this:  
//!   
//! ```QVS20
//! [Country ] [CityDataSubTable      ] [Population]  
//! [Slovenia] [1[Ljubljana][300000]1  
//!              [Koper    ][ 30000]1 ] [   2000000]  
//! [Italia  ] [1[Milano   ][400000]1  
//!              [Venezia  ][ 30000]1 ] [  60000000]  
//! ...  
//! ```
//!   
//! ## Row delimiter LF and sub-tables  
//!   
//! The basic row delimiter is LF. Not CR, not CRLF, but exactly LF.  
//! Every row must end with the row delimiter, especially the last row .  
//! There is a small performance problem with sub-tables here.  
//! Let me explain and come to the solution with this flow of thoughts.  
//! For sequential reading of `QVS20` files the inserted sub-tables are not a problem. It works well.  
//! If we read every field sequentially, we know when the sub-table starts and ends. Easy.
//! But sometimes we want to go very fast line by line and read only the first field for filtering. Because of sub-tables, the next LF is not always the start of a new row.  
//! **Let's solve this problem.**  
//! We can use again the fact that we know the different start and end delimiters.  
//! So between the end field and the new row we can put a different `row delimiter`.  
//! The first level row delimiter is conveniently LF.  
//! For the first sub-table the row delimiter changes to the string `1`.  
//! For the next depth level `2` and so on. The number is the depth of the sub-table.  
//! In any moment it will be clear what is the row delimiter for that explicit sub-table.  
//! Very important rule: every row, and the last row especially, must end with a row delimiter !  
//! We must take care to limit the row delimiter to only one byte. It means there cannot be a sub-table nested deeper than 9 levels.  And that is fine for this type of tabular format.  
//!
//! ## Schema  
//!   
//! The Schema is write always in 5 mandatory rows:  
//!
//! - 1st row - file type, table name and description  
//! - 2st row - data types  
//! - 3nd row - sub table schemas  
//! - 4nd row - additional data  
//! - 5rd row - column names  
//!
//! The Schema is mandatory.  
//! It can be included in the same `QVS20` file or can exist in an external `QVS20` file.  
//! That way is possible to have files with only data. Great for small packets of data.  
//! There is no other configuration needed to operate with qvs20.  
//!
//! ### Schema 1st row - file type, table name and description
//!
//! It is great to recognize the data from inside the data itself.  
//! The file name can change for various reasons and is not always coherent.  
//! There are 3 possible file types for qvs20:  
//!
//! 1. only schema - is marked with [S]
//! 2. only rows - is marked with [R]
//! 3. full (schema+rows) - is marked with [T]
//! 4. sub_table schema - is marked with [U], but it cannot be a standalone file
//!
//! The marker helps the parser to early recognize the file and its content.  
//! The table name is short and is used to assert that the separate TableRows file and the TableSchema file are really from the same Table.  
//! The description can be long if is needed. It is a string, therefore it is escaped.  
//!
//! ### Schema 2nd row - Data types
//!
//! The most important information for the parser is the data type of the column.
//! For example the strings must be unescaped, but the integers don't need it.
//! Example of the 2nd row with data type:  
//!   
//! ```QVS20
//! [String][Integer][Decimal][Float][Date][Time][DateTime][Bool][SubTable]  
//! ```
//!
//! ### Schema 3rd row - Sub table schemas  
//!
//! SubTables schemas are write in the 3rd row.  
//! For other column, the field is empty [].  
//!
//! ### Schema 4th row - Additional data  
//!   
//! Often programmers need to provide some additional info about the fields.  
//! Every use-case will need different data here.  
//! So this cannot be standardized.  
//! This data is optional, but the row is mandatory. So the parsing is clear!  
//! This additional data must be strings. And they must be correctly escaped in QVS20.  
//! Like any other String.
//!   
//! ### Schema 5th row - Column names  
//!   
//! The 5th row of the schema contains the column names.  
//! Example of 5th row with column names:  
//!   
//! ```QVS20
//! [City][Country][UN2018][CityPopulation][Area][MetropolitanPopulation][SomeDecimalNumber][SomeDate[SomeTime][SomeDateTime][SomeBool]  
//! ```  
//!
//! ## TableRows
//!
//! The actual data is write in TableRows. This can be in a separate file or in the same file after the schema.  
//!
//! ### TableRows 1st row - file type, table name
//!
//! There are 3 possible file types for qvs20:  
//!
//! 1. only schema - is marked with [S]
//! 2. only rows - is marked with [R]
//! 3. full (schema+rows) - is marked with [T]
//! 4. sub_table schema - is marked with [U], but it cannot be a standalone file
//!
//! The marker helps the parser to early recognize the file and its content.  
//! When TableRows are in separate file, the first row contains only file type and table name. No need for description here, because is already in the schema.  
//! With the table name we can check if the Schema and the Rows are from the same table when they are in separate files.  
//! When the schema and table are together in the same file, this row is not needed.  
//!
//! ## Versions  
//!   
//! Versions are named by the year. `QVS20` is the version from year 2020.  
//! If one day we make a new version we use the year of that version.  
//! For example in 2042 we will have the standard version and file extension  
//! `QVS42`.  
//!
//! ## File extension  
//!   
//! I want the file extension to be specific for the version of the standard.  
//! File extension and standard name are the same: `*.QVS20`  
//! It also looks unique and I hope nobody else will use it for something else.  
//!   
//! ## Implementation in Rust
//!   
//! A proposed standard without implementations is boring.  
//! In this project I also built rust libraries for QVS20 and published them on GitHub and crates.io.  
//! For rust developers they will have native libraries with source code.  
//! Rust is the best language to compile to WebAssembly/Wasm. So I will do that too.  
//! I will enjoy to have a `QVS20` parser inside the browser, to efficiently consume tabular data received from the server.  
//! I expect that authors of existent `csv parsers` will make an effort and create a similar library for `QVS20` in different languages. I am sure they can make it very very performant.  
//!
//! ## Popularity
//!
//! Sure the biggest problem for a standard is the popularization !  
//! If nobody uses it, than it is unusable. More people use it and more usable it becomes.  
//! Popularity is a vicious cycle. With time it accelerates - in either directions: up or down.
//! Integration in database tools, spreadsheet apps and client-server frameworks would boost the usability.  
//! It is really important for the standard to be popular.  
//! I hope the standard will be interesting to many and they will organically spread the word.  
//!
// endregion: lmake_md_to_doc_comments include README.md A //!

/// short macro `s!()` for &str.to_string or format!().
/// because that is so common.
/// Equivalents: s!(), x.to_string(), x.to_owned(),String::new(), format!()...
#[macro_export]
macro_rules! s {
    () => {
        String::new()
    };
    ($my_str: expr) => {
        $my_str.to_string()
    };
    ($literal: expr, $str_1: expr) => {
        format!($literal, $str_1)
    };
    ($literal: expr, $str_1: expr, $str_2: expr) => {
        format!($literal, $str_1, $str_2)
    };
    ($literal: expr, $str_1: expr, $str_2: expr, $str_3: expr) => {
        format!($literal, $str_1, $str_2, $str_3)
    };
    ($literal: expr, $str_1: expr, $str_2: expr, $str_3: expr, $str_4: expr) => {
        format!($literal, $str_1, $str_2, $str_3, $str_4)
    };
}

/// trim start of error for concatenation
#[macro_export]
macro_rules! err_trim {
    ($e: expr) => {
        $e.to_string().trim_start_matches("Error: ")
    };
}

// region: mod, extern and use statements
mod qvs20_reader_mod;
mod qvs20_table_mod;
mod qvs20_table_rows_mod;
mod qvs20_table_schema_mod;
mod qvs20_writer_mod;

// reexport objects for callers of the library
pub use qvs20_reader_mod::remove_src_loc;
pub use qvs20_reader_mod::Qvs20Error;
pub use qvs20_reader_mod::ReaderForQvs20;
pub use qvs20_table_mod::Table;
pub use qvs20_table_rows_mod::Row;
pub use qvs20_table_rows_mod::TableRows;
pub use qvs20_table_rows_mod::Value;
pub use qvs20_table_schema_mod::TableSchema;
pub use qvs20_table_schema_mod::DataType;
pub use qvs20_writer_mod::WriterForQvs20;
