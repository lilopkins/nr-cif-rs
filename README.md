# Network Rail CIF Parser

## Usage

### Reading the schedule

You can parse a CIF file into a Schedule database with the following code:

```no_run
use nr_cif::prelude::*;

use std::fs::File;

let f = File::open("full-or-partial.cif").expect("cannot read file");
let cif_result = parse_cif(f);
match cif_result {
    Ok(file) => {
        let mut schedule = ScheduleDatabase::new();
        let errors = schedule.apply_file(&file);
        log::info!("Complete.\n{schedule:#?}\nErrors: {errors:?}");
    },
    Err(e) => panic!("{e}"),
}
```

> **Note:** This does not always expose every field from the records.

### Parsing data in a raw manner

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

## Features

Feature | Purpose
--------|--------
`serde` | Enable serialization and deserialization on the objects.
`panic-on-first-error` | Panic if a parsing error is discovered. Mostly for testing.
