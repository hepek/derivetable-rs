DERIVETABLE
===========

This crate defines a procedural macro and some helper types to allow deriving
an indexed table from a struct defining a single row. This might be useful
for real time applications that require an indexed collection for fast lookups
and inserts.

Annotating different struct members generates Hash or BTree indexes and code
for querying and bookkeeping.

Example

```
use derivetable::*;

#[derive(Table, Debug)]
#[derivetable(Debug)]
struct Row {
    #[index]
    name: String,
    #[hindex]
    surname: String,
    height: f64,
    #[unique]
    ident: u64,
}
```

Each row field can be annotated by either `index`, `hindex` or `unique`.

`index` produces a BTree based ordered index on that field and two functions:
`get_by_<fieldname>` and `range_by_<fieldname>` that can be used to quickly
query an individual value or a range of values from the index.

`hindex` is similar to `index` but uses a hash map as an underlying structure.
It only supports querying for a single element, but might have better
performance as compared to BTree index. We also want to use this if our data type
is not `PartialOrd`.

`unique` enforces uniqueness on that field. This index is checked when
inserting new elements. Insert returns `Result<usize, usize>`. `Ok(idx)` is 
an internal index of inserted value. `Err(idx)` is returned when there exists a 
row in our table that has a `unique` field with the same value as the one we
are trying to insert.

Removing elements from the table invalidates internal indexes.
TODO: support removing mutliple items at once (e.g. remove all items returned from a range query).

The code above defines a new struct called `RowTable`. It roughly looks like this:

```
struct RowTable {
    ...
}

impl RowTable
{
    pub fn new() -> RowTable {...}
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Row> {...}
    pub fn insert(& mut self, row: Row) -> Result<usize,usize> {...}
    pub fn remove(&mut self, id: usize) -> Option<Row> {...}
    pub fn get_by_name<'a>(&'a self, idx_name: &String) -> impl DoubleEndedIterator<Item = (usize, &'a Row)> + 'a {...}
    pub fn range_by_name<'a, R>(&'a self, range: R) -> impl DoubleEndedIterator<Item = (usize, &'a Row) > + 'a
        where R : std::ops::RangeBounds<String>
    {...}
    pub fn get_by_surname<'a>(&'a self, idx_surname: &String) -> impl Iterator <Item = (usize, &'a Row)> + 'a {...}
    pub fn get_by_ident<'a>(&'a self, uidx_ident: &u64) -> Option<&'a Row> {...}
}
```


# Performance Measurement vs an In-memory sqlite3 Table

```
|             | Derivetable | Sqlite   | Speedup/Ratio   |
|=============|=============|==========|=================|
| 1m INSERT[s]| 1.075       | 22.310   | 20x             |
| LESSQ [ms]  | 2.573       | 109.18   | 42x             |
| EQQ [ms]    | 0.00101     | 0.0706   | 70x             |
| MEMORY [kB] | 835116      | 687940   | 1.21x           |
```
