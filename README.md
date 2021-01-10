DERIVETABLE
===========

Derive an indexed table struct from an annotated row struct.

Example

```
use derivetable::*;

#[derive(Table, Debug)]
#[derivetable(Debug)]
struct Row {
    #[index]
    name: String,
    #[index]
    surname: String,
    height: f64,
    #[unique]
    ident: u64,
}
```

|             | Derivetable | Sqlite   | Ratio   |
|=============|=============|==========|=========|
| 1m INSERT[s]| 1.075       | 22.310   | 20x     |
| LESSQ [ms]  | 2.573       | 109.18   | 42x     |
| EQQ [ms]    | 0.00101     | 0.0706   | 70x     |
| MEMORY [kB] | 835116      | 687940   | 1.21x   |
