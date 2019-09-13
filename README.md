# csvpivot
`csvpivot` is a tool for creating pivot tables from the command line. It's designed to be fast and memory-efficient so it can be used to
aggregate large datasets, and it's designed to be easy to use and easy
to debug.

## Table of Contents
* **[Installation](#installation)**
* **[Why Pivot Tables?](#why-should-you-use-pivot-tables)**
* **[Why `csvpivot`?](#why-csvpivot)**
* **[Why shouldn't you use `csvpivot`](#why-shouldnt-you-use-csvpivot)**
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
You can download binaries for Windows, Linux, and MacOS on
the [Releases](#https://github.coom/maxblee/csvpivot/releases/latest)
page of this repository.

Additionally, if you have Rust's package manager, Cargo, installed, you can run
```bash
$ cargo install csvpivot
```
which will compile `csvpivot` from source.

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
that syntax looks like in `csvpivot`:

```sh
$ csvpivot count test_csvs/layoffs.csv --rows=was_fired --val=id
```

That will print this out in your terminal:

```sh
,total
false,2
true,3
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
in `csvpivot`:

```sh
$ csvpivot sum test_csvs/layoffs.csv --rows=department --cols=was_fired --val=salary
```

Which will give you this output:

```csv
,true,false
engineering,75000,175000
sales,90000,85000
```

In other words, pivot tables provide convenient and easy-to-use ways to
aggregate datasets. 

## Why should you use `csvpivot`?

In a lot of cases, `csvpivot` isn't necessarily going to be any better
than existing tools for creating pivot tables. In the vast majority of
cases, you can easily do what `csvpivot` does using
[`pandas`](https://pandas.pydata.org/) in Python or using R.
And in a number of cases, you can use SQL or existing CSV toolkits like
[`csvtk`](https://github.com/shenwei356/csvtk) or [`xsv`](https://github.com/burntsushi/xsv). You can often use Excel, too, although Excel
doesn't offer good ways to help you document your work or sort your
pivot tables.

There are a couple of benefits to using `csvpivot` over these tools, though. 

`csvpivot` is easier to use than any CSV toolkit I'm aware of when it comes to creating pivot tables, because it's narrowly and specifically designed to create pivot tables. And it accepts input
from stanard input and filepaths and prints to standard output,
allowing you to pipe it into and out of other command-line programs.

`csvpivot` also makes it extremely easy to perform analyses on large datasets, including datasets that exceed the RAM on your computer.
I used the tool to analyze [the 80 GB ARCCOS dataset](https://www.washingtonpost.com/graphics/2019/investigations/dea-pain-pill-database/) the Washington Post acquired on my computer, which has 16 GB of RAM. In all, it took me about 10 minutes to create a CSV of the total number of oxycodone and
hydrocodone pills flowing into each ZIP code in
the United States between 2006 and 2012. That's far less time than
it would take me to figure out how to split the data into chunks
using `pandas`.

Beyond that, if you're already working at the command line, it can
simply be convenient to stay there.

## Why shouldn't you use `csvpivot`?

But `csvpivot` isn't always going to be the best tool to use.

Command-line programs are necessarily harder to configure than
libraries in programming languages, so if you need an aggregation
function that isn't supported by `csvpivot`, it's going to be easier
to use a data science library like `pandas` than it will be to configure
`csvpivot` for your use case. (As in, configuring `csvpivot` will
require you to make significant changes to the source code of
`csvpivot`.)

And `csvpivot` isn't designed for cleaning data. It has a limited number
of functions that will parse your data, but the parsing is mostly useful
for already well-formed data.

## Usage Guide
### Basic Usage
For basic syntax, I recommend that you use the help message provided with the binary:

```sh
$ csvpivot --help
csvpivot 0.1.0
Max Lee <maxbmhlee@gmail.com>
A tool for creating pivot tables from the command line. 
For more information, visit https://github.com/maxblee/csvpivot

USAGE:
    csvpivot [FLAGS] [OPTIONS] <aggfunc> --val <value> [--] [filename]

FLAGS:
        --day-first     In ambiguous datetimes, parses the day first. See
                        https://dateutil.readthedocs.io/en/stable/parser.html for details.
    -e                  Ignores empty/null values (e.g. "", NULL, NaN, etc.)
    -h, --help          Prints help information
    -i                  Infers the type/date format of type independent functions
        --no-header     Used when the CSV file does not have a header row
    -N                  Parses the type independent functions as numbers
    -t                  Fields are tab-separated (equivalent of setting delimiter to '\t')
    -V, --version       Prints version information
        --year-first    In ambiguous datetimes, parses the year first. See
                        https://dateutil.readthedocs.io/en/stable/parser.html for details.

OPTIONS:
    -c, --cols <columns>...    The name of the column(s) to aggregate on. Accepts String fieldnames or 0-indexed fields.
    -d, --delim <delim>        The delimiter used to separate fields (defaults to ',')
    -r, --rows <rows>...       The name of the index(es) to aggregate on. Accepts String fieldnames or 0-indexed fields.
    -v, --val <value>          The name of the field used to determine the value (e.g. id for most count
                               operations).\nAccepts String fieldnames or 0-indexed fields

ARGS:
    <aggfunc>     The function you use to run across the pivot table. 
                  The functions fit into three main categories: numeric, textual, and type independent. 
                  - Numeric functions parse records as numbers, raising an error if it can't identify a number. 
                        - `mean` computes a mean across the matching records 
                        - `median` computes the median of the matching records 
                        - `stddev` computes the sample standard deviation of the matching records 
                        - `sum` sums the values 
                  - Textual functions parse everything as text 
                        -`count` counts all of the individual records; it operates independently from the values 
                        -`countunique` counts all of the unique records. 
                        -`mode` determines the mode (the most commonly appearing value), in insertion order in the case of
                  a tie 
                  - Type independent functions have a default parsing method that can be overridden with the `-i` or
                  `-N` flags 
                        -`max` computes the maximum value. Defaults to strings (mainly for YYYYMMDD HHMMSS date formats) 
                        -`min` works identically to `max` but for computing the minimum value (or oldest date) 
                        -`range` computes the difference between `min` and `max`. Only works for valid numeric and date
                  formats. 
                   [values: count, countunique, max, mean, median, min, mode, range, stddev, sum]
    <filename>    The path to the delimited file you want to create a pivot table from
```

That should provide you with a decent overview of the usage of `csvpivot`. But let me provide a little bit more information.

The basic syntax of `csvpivot` is simple. Every command needs to have
a function and a values column connected to it. That values column
tells `csvpivot` which column it needs to apply an aggregation
function to. 

In addition, `csvpivot` needs a data source. This can either be explicitly typed after the name of the function, or it can be in the form of standard input. So the following commands are all equivalent:

```sh
$ csvpivot count mydata.csv --val id
$ cat mydata.csv | csvpivot count --val id
$ csvpivot count --val id < mydata.csv
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
* `0`: This will grab the first column, regardless of the name. (The numbers throughout `csvpivot` are 0-indexed to conform with standards in most programming languages.)
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

In case your data cannot be properly parsed by `csvpivot` using one of these functions, you can change
the encoding of your file on most Unix-based systems by using `iconv`. (The actual process of doing so may be a bit tricky, since figuring out
your file encoding is tricky and inexact, but `uchardet` and `chardetect` both work pretty well in most cases.) (Note: You will
likely have to install `uchardet` and `chardetect`. `chardetect`
requires Python and can be installed using `pip`, Python's package manager. `uchardet` can be installed using Homebrew in Mac or
apt for Linux.)

The functions that parse things as text are `count`, `median`, and `mode`. You can also technically use `min` and `max` to parse text,
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
cannot be parsed using `csvpivot`. (If you want to parse those from the
command line, I recommend `csvtk replace`.)

These functions are: `mean`, `median`, `stddev` (or the sample standard deviation), and `sum`.

With all of these functions, I have paid special attention to numerical
accuracy. `sum` and `mean` both use Decimal addition in order to avoid
truncation errors, while `stddev` uses [a numerically stable algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm). Furthermore, the mean and standard deviation algorithms are both tested
against the [Statistical Reference Datasets](https://www.itl.nist.gov/div898/strd/univ/homepage.html) from the Nation Institute of Standards and Technology.

#### Numerical *or* date functions

There are three algorithms designed to work with either numerical
data or with dates. They are the minimum, the maximum, and the range.

In the case of numerical data, the definitions for these terms should
be obvious. The minimum refers to the smallest number in the aggregation, the maximum refers to the largest number, and the range
refers to the difference between the minimum and the maximum.

**Note: In order to parse these numerical functions as numerical data,
you must type the `-N` flag.**

With dates, it's more complicated. The minimum date refers to the
earliest date in an aggregation, so an aggregation containing
the dates April 1, 2019 and March 31, 2019 would have a minimum of
March 31, 2019. The maximum date is then the most recent date, while
the range is the difference between the earliest date and the most
recent date, in days.

Dates get even more complicated than that because of the control you
have surrounding formatting. If you run either `min` or `max`
by default, you will get the string that evaluates to the lowest
value. This has the advantage of being considerably faster than converting all of the dates into datetimes. *However*, it is also less reliable. Dates will only sort properly if they are all in exactly the same format *and* if that format conforms to ISO standards
(i.e. some variant of YYYY-MM-DD or YY-MM-DD). All of which is to say you probably shouldn't run `min` or `max` under their string conditions unless you are only doing so to get a general sense of the date range of your data; you *really* trust the people who cleaned the data; or you tested all of the data against a regular expression
using a tool like `grep`.

Alternatively, `csvpivot` can convert dates into datetimes using one of two options.

First of all, you can pass the `-F` flag, along with a specification for how your datetimes are formatted.
This uses the string formatting options from Rust's `chrono` crate, which can be found 
[here](https://docs.rs/chrono/0.4.9/chrono/format/strftime/index.html).

This requires you to know how your dates are formatted and know or be willing to look up string formatting specifiers.
But it allows you to deal with a complicated set of date formatting options, and it runs faster than the second option.

You can also use the `-i` flag, which tries to automatically parse dates into datetimes. This uses Rust's
[`dtparse`](https://docs.rs/dtparse/1.0.3/dtparse/) library,
which is a port of Python's [`dateutil`](https://dateutil.readthedocs.io/en/stable/parser.html) parser. This will take
any date and try to convert it into a datetime object. 

If you are using the `-i` flag, you can also pass the `--year-first` or `--day-first` flags to `csvpivot` to alter how `csvpivot` interprets ambiguous dates like
`01-05-09`. These function like the dateutil parser's `dayfirst`
and `yearfirst` options, and I'd recommend visiting the dateutil documentation to learn more about how they function.

In order to get this date parsing behavior for `min` and `max`,
you need to use the `-i` flag or the `-F`. `range`, which *cannot* parse strings without serializing them
into datetime objects, uses the `-i` flag by default. But you can alternatively use the `-F` flag.

### Delimiters

You can also tell `csvpivot` to use something other than commas
as a field delimiter. By default, `csvpivot` will assume that files
ending with the `.tsv` or `.tab` extensions are tab-delimited,
while other files are assumed to be comma-separated. However, both of those can be overriden. You can select any other single ASCII character as a delimiter using the `-d` option, or you can use the
`-t` flag to choose to read tabs as the file dilimiter.

**Note: The file extension tool only works when `csvpivot` is
directly reading a file. If it is receiving tab-delimited data
from standard input, you need to use the `-t` flag or the `-d`
option.**

### Headers

If you don't have a header row, you can use the `--no-header` flag
to have `csvpivot` read the first row as a record, rather than as a header line. 

Alternatively, if you have a header row, but it is not on the first
line of your file, you can use `tail -n +` to have `csvpivot` read everything but the nth row. For instance, if the header row of your
CSV file `bad_csv.csv` is on the fifth line, you can type

```sh
tail bad_csv.csv -n +5 | csvpivot countunique -v 0
```

To count the number of unique values in the first column of your bad
CSV file.
### Null values

You can have `csvpivot` ignore empty values. If you use the `-e` flag,
`csvpivot` will skip past any cells that match (case- or whitespace-insensitively) to any of these strings:

* "": an empty string
* "na"
* "nan"
* "n/a"
* "none"
* "null"

As [this article](https://www.wired.com/2015/11/null/) eloquently
explains, this can be overly aggressive, so you should make sure
this is a reasonable approach for parsing your data. In particular,
I'd recommend spot-checking your data to see which points `csvpivot`
interprets as null before using the `-e` flag.

Which brings me to:

### Error handling
I've tried to make error handling clear and helpful in `csvpivot`.

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
that fields in `csvpivot` are zero-indexed.

* The third type of error you might see is a CSV error, from the CSV
parsing library `csvpivot` uses. Those errors look like this:
```sh
CSV error: record 1 (line: 2, byte: 597): found record with 4 fields, but the previous record has 1 fields
```
These errors can either come because of malformed CSVs or because
you forgot to specify the correct delimiter (for instance, forgetting
to use the `-t` flag when piping in a TSV file from standard input).

* Finally, you might get a parsing error that looks like this:
```sh
Could not parse record `NA` with index 167: Failed to parse as numeric type
```
This can be a sign that your file has some null or empty values in it,or that it is not as well-formatted as you might have hoped.

It can also be a sign that `csvpivot` is trying to parse your data in a different format than you expected (for instance, that it is trying to parse a bunch of strings as dates for the range function, when
you want it to parse everything as a number.)

These errors will all provide you with the string value of the record
`csvpivot` couldn't parse, the index of the record (where the first non-header record has an index of 0), and the type of data that it tried to parse your data into -- all of which should make it easier for you to debug.

(As a side note, I recommend pairing this utility with `xsv slice -i`, which prints out a row from a CSV file at a given line.)

## Contributors
So far, no one has directly contributed code to `csvpivot`.
(Note: Once you've used the tool, [you should change that](#developer-guide).) 

But a number of people have contributed in other ways.

In particular, the error handling I've used here comes
directly from
[this fantastic guide to error handling in Rust](https://blog.burntsushi.net/rust-error-handling/). I've also used the
design of [`xsv`](https://github.com/BurntSushi/xsv)
and the [`csv` crate in Rust](https://github.com/BurntSushi/rust-csv),
all by the same author, in a number of ways, while a number of other
guides were useful toward getting me to write code in Rust. I've tried to
document all of the guides and source code that helped me develop `csvpivot` in inline comments and docstrings within the source code.

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
Now that you've used `csvpivot`, do you want to help make it better?
I've concocted a guide (TODO) with some suggestions of things I'd like to see
improved. The guide is designed to allow people with no coding experience,
people who have written code in languages other than Rust, and people who
have written code in Rust to help. So don't by any means feel like you're not
qualified to improve this project. 

And I really mean that: If you can't even figure out what this program does
after reading the `--help` message and this guide, you can make `csvpivot`'s
documentation better. Just contact me and let me know what's confusing you.

## Contact Me
If you have any questions about `csvpivot` or if you have identified any bugs in the program or you want
to contribute to it, please send me an email at maxbmhlee@gmail.com or contact me through Twitter. 
I'm [@maxblee](https://twitter.com/maxblee). And if you wind up using `csvpivot` in any of your projects,
I'd love to know.