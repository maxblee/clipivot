# clipivot
`clipivot` is a tool for creating pivot tables from the command line. It's designed to be fast and memory-efficient so it can be used to aggregate large datasets, and it's designed to be easy to use and easy to debug.

## Table of Contents
* **[Installation](#installation)**
* **[Why Pivot Tables?](#why-should-you-use-pivot-tables)**
* **[Why `clipivot`?](#why-clipivot)**
* **[Why shouldn't you use `clipivot`](#why-shouldnt-you-use-clipivot)**
* **[Usage Guide](#usage-guide)**
    - **[Basic Usage](#basic-usage)**
    - **[Row names](#row-names)**
    - **[Functions](#functions)**
    - **[Delimiters](#delimiters)**
    - **[Headers](#headers)**
    - **[Null values](#null-values)**
    - **[Error handling](#error-handling)**
* **[Contributors](#contributors)**
* **[Developer Guide](#developer-guide)**
* **[Contact Me](#contact-me)**

## Installation
*Ideally*, you'll be able to download binaries for Windows, Linux, and MacOS on
the [Releases](#https://github.com/maxblee/clipivot/releases/latest)
page of this repository. However, I've had some difficulty getting Travis CI set up to
do that, so you may have to compile the program using cargo.

To do that, you'll need to 
use Rust's package manager, Cargo, and run
```bash
$ cargo install clipivot
```
which will compile `clipivot` from source.

## Why Pivot Tables?

At a basic level, pivot tables exist as a way to aggregate data
and extract meaning from datasets.

Say, for example, you have a list of salaries for employees. Each
record has a unique identifier for the employee, the employee's salary,
and the department the employee worked for. And let's say,
because I'm a journalist who's often bored by employee database examples,
there's also a field marking whether or not the employee was recently fired.

The dataset looks like this:
```csv
id,was_fired,salary,department
1,true,25000,sales
2,true,75000,engineering
3,false,175000,engineering
4,true,65000,sales
5,false,85000,sales
```
(You can see the file itself in `test_csvs/layoffs.csv`.)

With this data, you might want to know the number of employees
who were fired from the company, as well as the number employees who remain. You can do that easily with pivot tables. Here's what
that syntax looks like in `clipivot`:

```sh
$ clipivot count test_csvs/layoffs.csv --rows=was_fired --val=id
```

That will print this out in your terminal:

```sh
,total
true,3
false,2
```

Which tells you that three employees were fired and that two remain.

If you're familiar with SQL, you'll notice that this is similar
to running `GROUP BY` queries. In fact, you could run the same thing
I just did in SQL:

```sql
SELECT COUNT(id)
FROM my_table
GROUP BY was_fired;
```

Where pivot tables really provide an advantage over `GROUP BY` queries
is in their ability to allow you to control the output columns and rows
with ease.

If you want to find the total salary of the employees in the `layoffs.csv` dataset, aggregated both by the department and by
whether or not they were fired. You could do this in SQL:

```sql
SELECT SUM(salary)
FROM my_table
GROUP BY department, was_fired;
```

Which will create a table like this:

```csv
department,was_fired,sum
engineering,true,75000
engineering,false,175000
sales,false,85000
sales,true,90000
```

But you might want to set the values from the `was_fired` field as columns in the output, instead of as rows. That's trickier to do in SQL.
(I frankly don't know how to do it, but I wouldn't be surprised if it's
possible.)

With pivot tables, however, it's easy. Here's what that syntax looks like
in `clipivot`:

```sh
$ clipivot sum test_csvs/layoffs.csv --rows=department --cols=was_fired --val=salary
```

Which will give you this output:

```csv
,false,true
sales,85000,90000
engineering,175000,75000
```

In other words, pivot tables provide convenient and easy-to-use ways to
aggregate datasets. 

## Why should you use `clipivot`?

In a lot of cases, `clipivot` isn't necessarily going to be any better
than existing tools for creating pivot tables. In the vast majority of
cases, you can easily do what `clipivot` does using
[`pandas`](https://pandas.pydata.org/) in Python or using R.
And in a number of cases, you can use SQL or existing CSV toolkits like
[`csvtk`](https://github.com/shenwei356/csvtk) or [`xsv`](https://github.com/burntsushi/xsv). You can often use Excel, too, although Excel
doesn't offer good ways to help you document your work or sort your
pivot tables.

There are a couple of benefits to using `clipivot` over these tools, though. 

`clipivot` is easier to use than any CSV toolkit I'm aware of when it comes to creating pivot tables, because it's narrowly and specifically designed to create pivot tables. And it accepts input
from standard input and file paths and prints to standard output,
allowing you to pipe it into and out of other command-line programs.

`clipivot` also makes it easy to perform analyses on large datasets, including datasets that exceed the RAM on your computer.
I used the tool to analyze [the 80 GB ARCOS dataset](https://www.washingtonpost.com/graphics/2019/investigations/dea-pain-pill-database/) the Washington Post acquired on my laptop, which has 16 GB of RAM. In all, it took me about 10 minutes (with the data stored in an HDD external drive) to create a CSV of the total number of oxycodone and
hydrocodone pills flowing into each ZIP code in the United States between 2006 and 2012. And I didn't have to change any settings to get it to work, like I would've had to in `pandas`.

Beyond that, if you're already working at the command line, it can
simply be convenient to stay there.

## Why shouldn't you use `clipivot`?

But `clipivot` isn't always going to be the best tool to use.

Command-line programs are necessarily harder to configure than
libraries in programming languages, so if you need an aggregation
function that isn't supported by `clipivot`, it's going to be easier
to use a data science library like `pandas` than it will be to configure
`clipivot` for your use case. (As in, configuring `clipivot` will
require you to make significant changes to the source code of
`clipivot`.)

And `clipivot` isn't designed for cleaning data. It has a limited number
of functions that will parse your data, but the parsing is mostly useful
for already well-formed data.

## Usage Guide
### Basic Usage
For basic syntax, I recommend that you use the help message provided with the binary:

```sh
$ clipivot --help
clipivot 0.2.0
Max Lee <maxbmhlee@gmail.com>
A tool for creating pivot tables from the command line.
For more information, visit https://www.github.com/maxblee/clipivot

USAGE:
    clipivot [FLAGS] [OPTIONS] <aggfunc> --val <value> [--] [filename]

FLAGS:
    -A, --asc-rows      Displays the rows in sorted, ascending order (default is index order).
    -R, --desc-cols     Display column names in sorted, descending order (default is ascending)
    -D, --desc-rows     Displays the rows in sorted, descending order (default is index order).
    -e                  Ignores empty/null values ('', NULL, NaN, NONE, NA, N/A)
    -h, --help          Prints help information
    -I, --index-cols    Display column names in index order. Defaults to sorted, ascending order.
        --no-header     Skip the header row of the CSV file.
    -N                  Parse values as numeric data. This is only necessary for min, max, and minmax, which can parse
                        strings.
    -t                  Set the delimiter of the file to a tab.
    -V, --version       Prints version information

OPTIONS:
    -c, --cols <columns>...    The name of the column(s) to aggregate on. Accepts string fieldnames or 0-indexed fields.
    -d, --delim <delim>        The delimiter used to separate fields. Defaults to ','.
    -F <format>                The format of a date field (e.g. %Y-%m-%d for dates like 2010-09-21)
    -r, --rows <rows>...       The name of the index(es) to aggregate on. Accepts string fieldnames or 0-indexed fields.
    -v, --val <value>          

ARGS:
    <aggfunc>     The function you use to run across the pivot table.
                              - count counts the number of matching records.
                              - countunique counts the number of unique matching records.
                              - max returns the maximum value of the records given a specified data type.
                              - mean returns the mean.
                              - median returns the median value. Requires numeric data.
                              - min returns the minimum value of the records given a specified data type.
                              - minmax returns both the minimum and maximum values of the records, split by a
                  hyphen.
                              - mode returns the most commonly appearing value.
                              - range returns the difference between the minimum and maximum values. Returns the
                  number of days in the case of dates.
                              - stddev returns the sample standard deviation.
                              - sum returns the sum of the values. [values: count, countunique, max, mean, median,
                  min, minmax, mode, range, stddev, sum]
    <filename>    The path to the file you want to create a pivot table from
```

That should provide you with a decent overview of the usage of `clipivot`. But let me provide a little bit more information.

The basic syntax of `clipivot` is simple. Every command needs to have
a function and a values column connected to it. That values column
tells `clipivot` which column it needs to apply an aggregation
function to. 

In addition, `clipivot` needs a data source. This can either be explicitly typed after the name of the function, or it can be in the form of standard input. So the following commands are all equivalent:

```sh
$ clipivot count mydata.csv --val id
$ cat mydata.csv | clipivot count --val id
$ clipivot count --val id < mydata.csv
```

Finally, you can apply the `--cols` or `--rows` options to aggregate
by column. If you don't pass anything to those options, you will have
one row and/or one column named "total" that aggregates over
every single value in your dataset.

### Row names

There are a variety of names you can give to the `--rows`,
`--cols`, and `--val` options. 

Say we have a header row that looks like this:

```sh
col1,col2,col1,col3
```

In order to access the first column, we can type the following things:

* `col1`: This will grab the first column named `col1`
* `0`: This will grab the first column, regardless of the name. (The numbers throughout `clipivot` are 0-indexed to conform with standards in most programming languages.)
* `col1[0]`: This will grab the first column named `col1`

In order to access the third column, we can type the following things:

* `2`: Like `0` in the above example, this will grab the third column,
regardless of the name.
* `col1[1]`: This will grab the second column named `col1`.

Finally, for the `--rows` and `--cols` options, we can grab multiple values. There are several equivalent ways of doing this:

* `--cols=col1,col2`
* `-c=col1,col2`
* `-c col1 -c col2`
* `-c col1 col2`
* `--cols col1 col2`

### Functions

Once we know what columns we want to aggregate on, we need to choose a function. Different functions accept different types of data, so it's important to understand the distinction between them.

At a basic level, functions fit into three categories.

#### Text Functions

One category interprets every item as text. It will validate that your text is
valid UTF-8 but won't do any parsing on top of that. Because of that,
most data you encounter *should* be able to be parsed without error if you are using one of these methods.

In case your data cannot be properly parsed by `clipivot` using one of these functions, you can change
the encoding of your file on most Unix-based systems by using `iconv`. (The actual process of doing so may be a bit tricky, since figuring out
your file encoding is tricky and inexact, but `uchardet` and `chardetect` both work pretty well in most cases.) (Note: You will
likely have to install `uchardet` and `chardetect`. `chardetect`
requires Python and can be installed using `pip`, Python's package manager. `uchardet` can be installed using Homebrew in Mac or
apt for Linux.)

The functions that parse things as text are `count`, `median`, and `mode`. You can also technically use `min`, `max`, and `minmax` to parse text,
but that's primarily aimed at reading through dates, so we'll talk more
about that later.

#### Numeric Functions

Some functions only parse numeric data. The following formats all work
for numeric data, regardless of the aggregation function:

* `100`
* `1.2`
* `1e-6`
* `1E-6`
* `-1.5`

However, currency markers like dollar signs and thousands separators
cannot be parsed using `clipivot`. (If you want to parse those from the
command line, I recommend `csvtk replace`.)

These functions are: `mean`, `median`, `stddev` (or the sample standard deviation), and `sum`.

With all of these functions, I have paid special attention to numerical
accuracy. `sum` and `mean` both use Decimal addition in order to avoid
truncation errors, while `stddev` uses [a numerically stable algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm). Furthermore, the mean and standard deviation algorithms are both tested
against the [Statistical Reference Datasets](https://www.itl.nist.gov/div898/strd/univ/homepage.html) from the Nation Institute of Standards and Technology.

#### Numerical *or* date functions

There are four algorithms designed to work with either numerical
data or with dates. They are the minimum, the maximum, minmax (which outputs the minimum and maximum separated by a hyphen) and the range.

In the case of numerical data, the definitions for these terms should
be obvious. The minimum refers to the smallest number in the aggregation, the maximum refers to the largest number, the range
refers to the difference between the minimum and the maximum, and the minmax outputs the smallest number followed by a hyphen followed by the largest number.

**Note: In order to parse `min`,`max`, or `minmax` as numeric data,
you must type the `-N` flag.**

With dates, the minimum refers to the earliest date, so an aggregation containing
the dates April 1, 2019 and March 31, 2019 would have a minimum of
March 31, 2019. The maximum date is then the most recent date, while
the range is the difference between the earliest date and the most
recent date, in days.

In order to parse dates as date objects, you must pass the `-F` flag, along with a specification for how your datetimes are formatted.
This uses the string formatting options from Rust's `chrono` crate, which can be found 
[here](https://docs.rs/chrono/0.4.9/chrono/format/strftime/index.html).

### Sorting

With `clipivot`, you can choose how to sort the columns and rows of your pivot table -- by the order in which they appear,
in ascending, alphabetic order, or in descending, alphabetic order. By default, the columns will appear in sorted
ascending error, while the rows will appear in index order. However, you can override those defaults.

By using `-A` or `--asc-rows`, the rows will appear in ascending order; by using `-D` or `--desc-cols`, they will appear in descending order. By using `-R` or `--desc-cols`, the columns will appear in descending order; by using `-I` or `--index-cols`, they will appear in the order in which they appear.

### Additional Information

The broad definitions of functions are provided in the help message. However, there are a few things I should clarify here:

- `clipivot` technically allows you to parse the `min`, `max`, and `minmax` functions as strings, or text. (In fact, this is the default.) This is almost completely intended to speed up the processing of dates in formats like YYYY-MM-DD that sort alphabetically. 
- In cases where there is more than 1 true mode, the mode algorithm here simply returns the value that first reached
the maximum number of occurrences (so, if you have a set of values "a, b, b, a", it would return "b", because the second occurrence of "b" happened earlier than the second occurrence of "a.")
- The standard deviation returns the *sample* standard deviation.

### Delimiters

You can also tell `clipivot` to use something other than commas
as a field delimiter. By default, `clipivot` will assume that files
ending with the `.tsv` or `.tab` extensions are tab-delimited,
while other files are assumed to be comma-separated. However, both of those can be overridden. You can select any other 
single-byte UTF-8 character as a delimiter using the `-d` option, or you can use the
`-t` flag to choose to read tabs as the file dilimiter.

**Note: The file extension tool only works when `clipivot` is
directly reading a file. If it is receiving tab-delimited data
from standard input, you need to use the `-t` flag or the `-d`
option.**

### Headers

If you don't have a header row, you can use the `--no-header` flag
to have `clipivot` read the first row as a record, rather than as a header line. 

Alternatively, if you have a header row, but it is not on the first
line of your file, you can use `tail -n +` to have `clipivot` read everything but the nth row. For instance, if the header row of your CSV file `bad_csv.csv` is on the fifth line, you can type

```sh
tail bad_csv.csv -n +5 | clipivot countunique -v 0
```

To count the number of unique values in the first column of your bad
CSV file.

### Null values

You can have `clipivot` ignore empty values. If you use the `-e` flag,
`clipivot` will skip past any cells that match (case- or whitespace-insensitively) to any of these strings:

* "": an empty string
* "na"
* "nan"
* "n/a"
* "none"
* "null"

As [this article](https://www.wired.com/2015/11/null/) eloquently
explains, this can be overly aggressive, so you should make sure
this is a reasonable approach for parsing your data. In particular,
I'd recommend spot-checking your data to see which points `clipivot`
interprets as null before using the `-e` flag.

Which brings me to:

### Error handling
I've tried to make error handling clear and helpful in `clipivot`.

In all, there are four errors you might wind up seeing.

* The first is a simple IO error. It looks like this:
```sh
No such file or directory (os error 2)
```
If you see this error, it probably means you had a typo
when you tried to spell the name of your file.

* The second type of error you might see is a configuration error. 
Configuration errors can take a number of forms, each of which should
have a detailed error message providing you with specific information
debugging information. One example looks like this:
```sh
Could not properly configure the aggregator: Column selection must be between 0 <= selection < 42
```
If you see that error, there's a decent chance you simply forgot
that fields in `clipivot` are zero-indexed.

* The third type of error you might see is a CSV error, from the CSV
parsing library `clipivot` uses. Those errors look like this:
```sh
CSV error: record 1 (line: 2, byte: 597): found record with 4 fields, but the previous record has 1 fields
```
These errors can either come because of malformed CSVs or because
you forgot to specify the correct delimiter (for instance, forgetting
to use the `-t` flag when piping in a TSV file from standard input).

* Finally, you might get a parsing error that looks like this:
```sh
Could not parse record `NA` with index 167: Failed to parse as numeric
```
This can be a sign that your file has some null or empty values in it,
or that it is not as well-formatted as you might have hoped.

It can also be a sign that `clipivot` is trying to parse your data in a different format than you expected (for instance, that it is trying to parse a bunch of strings as dates for the range function, when
you want it to parse everything as a number.)

These errors will all provide you with the string value of the record
`clipivot` couldn't parse, the index of the record (where the first non-header record has an index of 0), and the type of data that it tried to parse your data into -- all of which should make it easier for you to debug.

(As a side note, I recommend pairing this utility with `xsv slice -i`, which prints out a row from a CSV file at a given line.)

## Contributors

The design for the sorting comes from [this issue](https://github.com/maxblee/clipivot/issues/2).

The error handling I've used here comes directly from
[this fantastic guide to error handling in Rust](https://blog.burntsushi.net/rust-error-handling/). I've additionally
used the shell scripts along with other design components and code snippets from [`xsv`](https://github.com/BurntSushi/xsv)
and the [`csv` crate in Rust](https://github.com/BurntSushi/rust-csv).

A number of other
guides were useful toward getting me to write code in Rust. I've tried to
document all of the guides and source code that helped me develop `clipivot` in inline comments and docstrings within the source code.

Other CSV toolkits also helped me design this program. The most direct
connection between these toolkits is probably the approach I've taken to
parsing null values, which is directly inspired by the approach taken
by the [`agate`](https://agate.readthedocs.io/en/1.6.1/) library
in Python, which serves as the backbone of [`csvkit`](https://csvkit.readthedocs.io/en/latest/). 

And I'm sure there are other, subtler ways in which existing CSV
toolkits have inspired the design of this project. The main toolkits I use are the previously mentioned `xsv` and the excellent [`csvtk`](https://github.com/shenwei356/csvtk). If you're
interested in doing more things with CSV files from the commmand line,
I strongly recommend them both.

And finally, the CSV files I've used to validate the numerical accuracy
of the mean and standard deviation functions (in `tests/test_numerical_accuracy.rs`) are from the [Statistical Reference Datasets](https://www.itl.nist.gov/div898/strd/univ/homepage.html) from the Nation Institute of Standards and Technology.

## Developer Guide
If you want to make changes to `clipivot`, I recommend you look at [the developer guide](https://docs.rs/clipivot/0.1.0/index.html), which provides an overview of the design of the code along with some suggestions of things I'd like to see
improved. The guide is designed to allow people with no coding experience,
people who have written code but haven't written any Rust, and people who
have written code in Rust to help. So don't by any means feel like you're not
qualified to improve this project. 

I've concocted a developer guide. Currently, you need to have Rust's package manager, Cargo, installed to see it. Simply
type
```bash
$ cargo doc --open
```
and a browser will open displaying the guide. Eventually, I want to have the developer guide published on
Rust's documentation website, [docs.rs](https://docs.rs), but I haven't figured out how to get that to work.
(If you know how to get it to display an API for a binary, let me know!)

In the meantime, here are the things I'd generally like to see improved with the tool. I've divided them into
things that I think you'd need to have coding experience in order to address, and things that don't require any
programming experience whatsoever. 

### Requires programming experience
- Performance: I've tried to design `clipivot` to be reasonably performant, but I'm sure there
are places where performance could be optimized. If you have any suggestions, I'd love to hear them.
(Note: I'm aware that there are technically faster algorithms for computing median than the one I
wound up with, the [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
in Rust's standard library. The reason I chose the `BTreeMap` is that it is well-suited for
adding items from a stream and it is more memory efficient than other algorithms I'm aware of.
But let me know if you're aware of a way to improve the speed of the median computation
while maintaining the best case memory efficiency of `BTreeMap`.)
- Coding style: This is my first project in Rust, so I'm sure there are parts of the code
that are not idiomatic in Rust or that are poorly structured.
- Testing: I think I've included fairly decent testing for this tool, but I'm sure there are places
where my testing can improve.
- Coverage Testing: If you're familiar with coverage testing schemes in Rust, I'd love your help.
Right now, I don't have any coverage testing on this crate because the one coverage testing tool
I've gotten working in stable Rust panics when I include property-based tests from Rust's
`proptest` crate.
(This is because of a bug in Rust's compiler; see more [here](https://github.com/xd009642/tarpaulin/issues/161).)
- Continuous Integration: Thanks to [two](https://github.com/japaric/trust) [great](https://github.com/BurntSushi/xsv)
templates, I managed to get continuous integration working in Travis CI for two version ins of Linux, one version of OSX,
and one version of Windows. However, some versions I tried to deploy failed 
(they're currently commented out in the .travis.yml file). If anyone wants to help get those working or wants to add support
for other environments, I would really appreciate it.

### Doesn't require programming experience
- Bugs: If something in this program doesn't work like you think it's supposed to, please let me know.
- Error handling: I've tried to make error handling as clear and helpful as possible, so if an error
message you get from `clipivot` confuses you, let me know and I'll do what I can to fix it.

In particular, pretty much nothing you run should ever result in what Rust calls a "panic" -- basically an unanticipated,
fast exit from a program. Panics look something like:

```sh
thread 'main' panicked at 'explicit_panic', src/main.rs:5:1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
The only exceptions I can think of are that some of the mathematical operations can techinically
result in overflows, and that all of the algorithms can potentially cause you to run out of memory.
But both of those examples should be exceptionally rare (even when dealing with datasets larger than your RAM),
so if you ever run into a panic, please send me a bit of information about the query you ran so I can fix this.

### Development Environment
In order to contribute code, first clone the repository to install the source code:

```sh
$ git clone https://github.com/maxblee/clipivot
```
Then, make changes to the code and/or add/change tests, and then run

```sh
$ cargo test
```

to run tests.
### Formatting
In addition, I use `clippy` to lint code and `rustfmt` to automatically format code.

To install them, type
```sh
$ rustup update
$ rustup component add rustfmt --toolchain stable
$ rustup component add clippy --toolchain stable
```
And from there, you can run `rustfmt` with
```sh
$ cargo fmt --all
```
and `clippy` with
```sh
$ cargo clippy -- -A clippy::ptr_arg
```
**Note that I am ignoring the `clippy::ptr_arg` warning, which raises a warning when you put a `&Vec<T>` into a function call.**

## Contact Me
If you have any questions about `clipivot` or if you have identified any bugs in the program or you want
to contribute to it, please send me an email at maxbmhlee@gmail.com or contact me through Twitter. 
I'm [@maxblee](https://twitter.com/maxblee). And if you wind up using `clipivot` in any of your projects,
I'd love to know.
