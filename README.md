# Network Rail CIF Parser

## Usage

You can parse a CIF file into a records array with the following code:

```no_run
use nr_cif::prelude::*;

use std::fs::File;

let f = File::open("full-or-partial.cif").expect("cannot read file");
let cif_result = parse_cif(f);
match cif_result {
    Ok(file) => {
        for record in file.records() {
            // do something with each record
        }
    },
    Err(e) => panic!("{e}"),
}
```

This can then be processed further manually.
