# EXAMPLES AND DEVELOPMENT

## examples

Use this examples to learn how to use the qvs21 format and library.  

[//]: # (auto_md_to_doc_comments segment start A)

## cargo make - for non-trivial or multi-line commands

Cargo-make is a utility to write simple "scripts" to use in development.  
I use it to store in one place all the commands that are frequently in use in development.  <https://github.com/sagiegurari/cargo-make>

```bash
# install cargo plugin
cargo install --force cargo-make
# reads the Makefile.toml and shows the prepared scripts:
clear; cargo make
# for example
clear; cargo make publish_to_web - can have many steps to copy, upload, tag, stop/start server  
clear; cargo make doc - have many steps to prepare the md, doc comments and finally generate the documentation
...and more
```
