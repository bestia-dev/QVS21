[comment]: # (lmake_md_to_doc_comments segment start A)

# QVS21

[comment]: # (lmake_readme cargo.toml data start)
**Modern replacement for csv for the year 2021**  
***version: 0.0.8  date: 2021-07-16 author: [bestia.dev](https://bestia.dev) repository: [GitHub](https://github.com/bestia-dev/QVS21)***  

[comment]: # (lmake_readme cargo.toml data end)

[comment]: # (lmake_lines_of_code start)
[![Lines in Rust code](https://img.shields.io/badge/Lines_in_Rust-1574-green.svg)](https://github.com/bestia-dev/QVS21/)
[![Lines in Doc comments](https://img.shields.io/badge/Lines_in_Doc_comments-612-blue.svg)](https://github.com/bestia-dev/QVS21/)
[![Lines in Comments](https://img.shields.io/badge/Lines_in_comments-211-purple.svg)](https://github.com/bestia-dev/QVS21/)
[![Lines in examples](https://img.shields.io/badge/Lines_in_examples-954-yellow.svg)](https://github.com/bestia-dev/QVS21/)
[![Lines in tests](https://img.shields.io/badge/Lines_in_tests-812-orange.svg)](https://github.com/bestia-dev/QVS21/)

[comment]: # (lmake_lines_of_code end)
  
[![crates.io](https://img.shields.io/crates/v/qvs21.svg)](https://crates.io/crates/qvs21) [![Documentation](https://docs.rs/qvs21/badge.svg)](https://docs.rs/qvs21/) [![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/qvs21.svg)](https://web.crev.dev/rust-reviews/crate/qvs21/) [![Lib.rs](https://img.shields.io/badge/Lib.rs-rust-orange.svg)](https://lib.rs/crates/qvs21/) [![Licence](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bestia-dev/qvs21/blob/master/LICENSE)  

My proposed format for import/export of 2 dimensional database tables.  
  
## Difference between QVS20 and QVS21
  
`QVS` stands for "sQuare brackets Separated Values".  
The number `20` is the year of the first version: 2020.  
The number `21` is the year of the second version: 2021.
The second version has everything from the first version plus added support for SubTables.  
It is not an update, but a more complex version.  
For most projects it is enough to use the QVS2020. The code is smaller, faster, simpler.  
Only if there is absolute need for SubTables the QVS21 must be used.  
Except for SubTables these versions are 100% compatible.  
Find the repository for QVS20 here: <https://github.com/bestia-dev/QVS20>.

## Data type SubTable
  
Inside one cell of the table is possible to insert a whole sub-table.  
Just like that. Nothing changes, no special escaping, because we have start and end delimiters.  
So we can represent also hierarchical data, if it is really needed.  
Still the primary goal of the standard is tabular data.  
Yes, but one tiny thing must change: the row delimiter of the sub-table is not \n anymore.  
It is the number that represent the depth of the sub-table: 1, 2, 3,...  
It must be only one byte. This format is not really great for hierarchies deeper than 9 levels.  

Example of CountryTable:  

```QVS21
[Country][Population]  
[Slovenia][2000000]  
[Italia][60000000]  
[Croatia][4000000]  
```
  
Now we want to insert the data of the cities, but hierarchically as a sub-table.  
Example of CityDataTable for Slovenia:  

```QVS21
[City][MetropolitanPopulation]  
[Ljubljana][300000]  
[Koper][30000]  
```

Together table and sub-table looks like this:  
The sub-table row delimiter is changed to "1".  
For easy parsing, the sub-table starts with the new row delimiter.  
  
```QVS21
[Country][CityDataSubTable][Population]  
[Slovenia][1[Ljubljana][300000]1[Koper][30000]1][2000000]  
[Italia][1[Milano][400000]1[Venezia][30000]1][60000]  
```

This is not very human readable, because the lack of spaces.  
But you can visualize it like this:  
  
```QVS21
[Country ] [CityDataSubTable      ] [Population]  
[Slovenia] [1[Ljubljana][300000]1  
             [Koper    ][ 30000]1 ] [   2000000]  
[Italia  ] [1[Milano   ][400000]1  
             [Venezia  ][ 30000]1 ] [  60000000]  
...  
```
  
## Row delimiter LF and sub-tables  
  
The basic row delimiter is LF. Not CR, not CRLF, but exactly LF.  
Every row must end with the row delimiter, especially the last row .  
There is a small performance problem with sub-tables here.  
Let me explain and come to the solution with this flow of thoughts.  
For sequential reading of `QVS21` files the inserted sub-tables are not a problem. It works well.  
If we read every field sequentially, we know when the sub-table starts and ends. Easy.
But sometimes we want to go very fast line by line and read only the first field for filtering. Because of sub-tables, the next LF is not always the start of a new row.  
**Let's solve this problem.**  
We can use again the fact that we know the different start and end delimiters.  
So between the end field and the new row we can put a different `row delimiter`.  
The first level row delimiter is conveniently LF.  
For the first sub-table the row delimiter changes to the string `1`.  
For the next depth level `2` and so on. The number is the depth of the sub-table.  
In any moment it will be clear what is the row delimiter for that explicit sub-table.  
Very important rule: every row, and the last row especially, must end with a row delimiter !  
We must take care to limit the row delimiter to only one byte. It means there cannot be a sub-table nested deeper than 9 levels.  And that is fine for this type of tabular format.  

### Schema 3rd row - Sub table schemas  

SubTables schemas are write in the 3rd row.  
For other column, the field is empty [].  

## File extension  
  
I want the file extension to be specific for the version of the standard.  
File extension and standard name are the same: `*.QVS21`  
  
[comment]: # (lmake_md_to_doc_comments segment end A)

## Read also the separate XXX.md files

I use the same README.md file for GitHub, Crates.io and docs.rs.  
So I cannot include the specific information that are not common to all 3 purposes. For that reason I have separate XXX.md files:  

- DEVELOPMENT.md - information and instruction for development
- CHANGELOG.md - what changed between versions
- TODO.md - reminder of what is in plan to do
