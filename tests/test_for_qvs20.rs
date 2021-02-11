//! test_for_qvs20
use qvs20::*;

#[test]
fn t10_sub_table() {
    let s = r"[T][table_name][description]
[String][SubTable][String]
[][1[U][vers][description]1[String]1[]1[]1[ver]1][]
[blue][red][x]
[name][vers][desc]
[name_1][1[1.0]1[1.1]1[2.1]1[2.2]1][desc_2]
[name_3][1[1.0]1[1.1]1[2.1]1[2.2]1][desc_4]

";
    let err = Table::from_qvs20_str_with_schema(&s).unwrap_err();
    assert_eq!(
        remove_src_loc(err),
        "Error: row 1 The row delimiter has more than 1 byte. ./file.qvs20:6:44"
    );
}
