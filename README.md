# csvpivot
A tool for creating pivot tables from the command line.

## Contributors
So far, no one has contributed to `csvpivot` other than me.
(Note: Once you've used it,
[you should change that](#developer-guide).)

But that's not to say that it would be possible without a bunch of other people.

I'll leave out a lot of the specifics; if you care to figure them out,
contact me or read through the comments in the source code and the
Cargo.toml file in the root directory. But I specifically wanted to make
a note of the `agate` library, which motivated how I determined whether or not
a cell was null, and
[this fantastic guide to error handling in Rust](https://blog.burntsushi.net/rust-error-handling/),
which formed the basis for the error handling of this program.

## Table of Contents
* **[What is this?](#what-is-this-the-pitch)**
* **[What isn't this?](#what-isnt-this-the-anti-pitch)**
* **[Usage Guide](#usage-guide)**
    - **[Installation](#installation)**
    - **[Using it in practice](#a-practical-guide)**
    - **[Basic Usage](#basic-usage)**
    - **[Supported Functions](#supported-functions)**
    - **[Other Options](#other-options)**
* **[Developer Guide](#developer-guide)**
* **[Contact Me](#contact-me)**

## What is this? (The pitch)

### The simple answer (for people with minimal experience using command-line tools)
Pivot tables are extremely useful for processing and extracting valuable information out of data,
as you'll see shortly in a tutorial. They're designed to take disparate records and aggregate them
in meaningful ways that can help you understand your data better and that can help you find stories
within your data.

This tool is designed to provide an easy and fast way to create pivot tables from the command line.
Its primary advantage over other pivot table tools, like Excel or `pandas` in Python, lies in how it handles
large datasets and in the fact that it's designed to work nicely with other command-line tools.
Depending on your preferences, it can also simply be a convenient tool.

### The more complicated answer
The main reason to use `csvpivot` is that it reads standard input in addition to files and that it outputs everything
into standard output in a standardized CSV format. That means that it interfaces well with other CSV tools (I'll show
some examples of that in a bit), allowing you to take output from another CSV tool or to use the results from this tool
as the input for another CSV tool. 

In addition, `csvpivot` is fast and should be able to handle extremely large datasets, even those that exceed the RAM
of your device. (However, it does hold a significant amount of information in memory, especially when calculating
the mode and median, so it is possible for you to run into memory issues when using it.)

And finally, it's accurate. With the exception of calculating the standard deviation, all of the numeric methods
parse values as decimals, avoiding roundoff errors. And the standard deviation uses
[Welford's algorithm](https://www.johndcook.com/blog/standard_deviation/) to avoid cancellation errors.

## What isn't this? (The anti-pitch)
* `csvpivot` is a tool, not a toolkit. There are too many good CSV toolkits out there for me to be able to justify
creating a new one. However, `csvpivot` *is* designed to play nicely with other command-line CSV toolkits. In
just a short bit, I'll show you some of my favorite toolkits and how you can use `csvpivot` in
conjunction with them. (You'll also get a brief introduction to them in the tutorial shortly.)

* `csvpivot` is not flexible. I've tried to anticipate the most common aggregation methods, from counting to calculating
the standard deviation on a column of values given a set of constraints. But if the available aggregation methods do not
support your particular use case, you should probably use `SQL` or a data science library like `pandas`.

* `csvpivot` is not going to outperform `SQL`. While I've tried to keep the program reasonably fast, it will not reach
the speeds of `SQL` performance. Queries should be easier to write, however.

* `csvpivot` is not a publication tool. Finding decent ways to aggregate data in a way that is reproducible for a large
number of datasets and a large number of stories is not easy. So a lot of the time, you will have to clean the CSV
files after running it through this program. However, I have tried to design the program to operate predictably so
cleaning data should be somewhat easy.

## Usage Guide
### Installation
TODO
### A practical guide
Before I get started in the technicalities of using this tool, I want to show off what it does, providing you with
a little bit of a sense of why you should use pivot tables, what the syntax of this tool looks like, and how you
can use it with existing CSV pivot tools.

We'll be making extensive use of piping with powerful CSV toolkits, so if you want to follow along with this guide,
you'll need to install [csvtk](https://github.com/shenwei356/csvtk) and [xsv](https://github.com/BurntSushi/xsv),
both of which are extremely useful CSV command-line toolkits. In addition, I'd recommend you take a look at
[miller](https://github.com/johnkerl/miller) and [csvkit](https://csvkit.readthedocs.io/en/latest/scripts/csvclean.html).

So with all of that said, let's take a look.

We're going to start by looking at donation data for President Trump's 2017 Inauguration. *The Washington Post*
published [the data](https://github.com/washingtonpost/data-inaugural-committee) we're going to use and wrote
[the story](https://www.washingtonpost.com/politics/2018/12/14/how-money-flowed-into-out-trumps-inaugural-committee/?utm_term=.54dbde88e0a7)
that forms the basis for this part of our analysis. Specifically, here's the part of the story we'll be looking at:

> Of the money that was raised, more than half came from individual donors, according to data compiled by CBNC’s 
Christina Wilkie and the Center for Responsive Politics. No single entity gave more than casino mogul Sheldon Adelson,
 who ponied up $5 million.
 
How do we validate these findings? Well, if we look at our data, there's a field called `entity_type` that refers to
the type of donation (whether it came from an individual donor or an organization, primarily). And there's a `donation` field
that corresponds to the total amount of each individual donation. So what we need is a sum of the `donation`
field aggregated by `entity_type`. But before we do that in `csvpivot`, let's take a look at that `donation` field:

```bash
$ xsv select donation | xsv slice -e 4 inaugural_donations.csv
donation
$100.00
"$75,000.00"
"$100,000.00"
$100.00
```
(The command above simply selects the `donation` column and then displays the first four rows of that column.

There are a few problems with this data that might or might not stick out to you. In order to calculate the sum of
something, we need to add up a bunch of numbers. But it's hard for a computer to read "$100,000.00" as a number.
So we need to strip the characters "$" and "," from all records in the `donation` field. (You might also think
that we need to strip the quotation marks, but `csvpivot` does that by default.) Here's how you'd do that with `csvtk`:
```bash
$ csvtk replace -p '(\$|,)' -r '' -f donation inaugural_donations.csv | xsv select donation | xsv slice -e 4
donation
100.00
75000.00
100000.00
100.00
```
That's better! Now, we can combine the `csvtk` method with `csvpivot`, and finally sort the resulting CSV file
to show us whether individual donors did, in fact, donate more than organizational or corporate donors:
```bash
$ csvtk replace -p '(\$|,)' -r '' -f donation inaugural_donations.csv | csvpivot sum -r 2 -v 4 | xsv sort -N -s total -R
,total
IND,59538730.00
ORG,47171478.29
MOC,5100.00
```
And sure enough! Individual donors spent about $2 million more than organizational donors did.

Now, for the fun of it, we'll make sure Sheldon Adelson donated $5 million, more than any other individual or entity.
Intuitively, this should seem easy; it's the same thing we just did, but we're aggregating by `name/org` instead of
by `entity_type`:
```bash
csvtk replace -p '(\$|,)' -r '' -f donation inaugural_donations.csv | csvpivot sum -r 3 -v 4 |
> xsv sort -N -s total -R | xsv slice -e 4
,total
"ADELSON, SHELDON, G",5000000.00
AT&T,2082483.43
BOEING COMPANY,1000000.00
KUMAR FAMILY LTD,1000000.00
```
And, indeed, Sheldon Adelson donated more than any other individual or entity, at $5 million.

`csvpivot` can do a lot more. You can decide to aggregate on multiple rows, which will separate the values from the
individual rows with a "$." separator, and you can decide to aggregate on columns, which will
act similarly to aggregating on rows. (That is, you'll get a new column for each unique item in that column(s),
and, if you select multiple columns, the unique value in each column will be marked with a "$." separator.)

Finally, there are a host of other functions `csvpivot` supports in addition to sum. We'll take a look at those next.
### Basic Usage
The best place to start for a general understanding of `csvpivot` is

```bash
$ csvpivot --help
csvpivot 0.1.0
Max Lee <maxbmhlee@gmail.com>
A tool for creating pivot tables from the command line. 
For more information, visit https://github.com/maxblee/csvpivot

USAGE:
    csvpivot [FLAGS] [OPTIONS] <aggfunc> --val <value> [--] [filename]

FLAGS:
    -e                 Ignores empty/null values (e.g. "", NULL, NaN, etc.)
    -h, --help         Prints help information
    -i                 Infers the type/date format of type independent functions
        --no-header    Used when the CSV file does not have a header row
    -N                 Parses the type independent functions as numbers
    -V, --version      Prints version information

OPTIONS:
    -c, --cols <columns>...    The name of the column(s) to aggregate on. Currently must be 0-indexed integers
    -d <delimeter>             The field separator in the CSV file. Defaults to ',' if unspecified
    -q <quotechar>             The quote character to parse the CSV records with (defaults to '"' if unspecified)
    -r, --rows <rows>...       The name of the index(es) to aggregate on. Currently must be 0-indexed integers
    -v, --val <value>          The name of the field used to determine the value (e.g. id for most count operations).
                               Currently must be 0-indexed integers

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
                   [values: count, countunique, mean, median, mode, stddev, sum]
    <filename>    The path to the delimited file you want to create a pivot table from
```
That should provide you with a basic understanding of the tool, assuming you know what a pivot table is.
(If you're familiar with pivot tables but can't figure out how to get a simple one to work,
let me know what's confusing you and I'll try to make the help message clearer.)

But let's say you don't know what a pivot table is.

At its most basic level, a pivot table is a method for aggregating data based on the value of cells
at different column(s) and applying some function based on the value at another column.
If you're familiar with SQL, you may have created a simple pivot table already:

```sql
SELECT COUNT(id)
FROM my_table
GROUP BY name;
```
That SQL command aggregates the number of records by the `name` field and counts the number of records
for each name appearing in your table.

The same basic command can be performed easily in `csvpivot`:
```bash
$ csvpivot count my_table.csv -r 1 -v 0
```
assuming the `id` field is the first column in the dataset and the `name` field is the second.
(Note: `csvpivot` uses 0-indexed field naming conventions. So if a given field is the first field
in the CSV file, you describe it with "0", if it's the second field in the file, you describe it with a "1", etc.)

But pivot tables extend a bit beyond that. In addition to allowing you to aggregate fields into rows
in your new datasets, you can aggregate them by columns. This especially comes in handy with fields that have a
specified set of possible values, like boolean TRUE/FALSE fields.

Let's see a mock example of this. Here's our data -- we'll call it `layoffs.csv` -- which is a completely made up
dataset displaying a unique identifier for each person working in Company A, that person's salary, the department
they work in, and whether or not they were just fired. Here's what it looks like:

```csv
id,was_fired,salary,department
1,true,25000,sales
2,true,75000,engineering
3,false,175000,engineering
4,true,65000,sales
5,false,85000,sales
```

Now, let's say we want to know the total number of people who were fired from this company, aggregated
by the department they worked in. We'll set the values column to 0 because each row refers to a unique record,
but because we're just working with counts we can set it to any field, from 0 to 3.

The other fields matter more. In order to aggregate by `department`, we'll set the rows of our pivot table
to 3 (since `department` is the fourth field and `csvpivot` uses 0-indexed columns). And we'll set the
columns for our pivot table to 1, for the `was_fired` field.

Let's see that in action.

```bash
$ csvpivot count layoffs.csv -r 3 -c 1 -v 0
,true,false
sales,2,1
engineering,1,1
```

And now we can see that there were three people in the sales department, that two were fired and one was not;
and that there were two people in the engineering department, one of whom was fired.

Instinctively, you should be able to see the advantage of using pivot tables. Immediately, you can see not only
*how many* people were fired by each department, but you can see how many were not. 

And your data is set up so you can easily compute the percentage of employees from each department that were fired
(with another program). To output the data as a CSV file, you'd simply type:
```bash
$ csvpivot count layoffs.csv -r 3 -c 1 -v 0 > new_layoffs.csv
```
which would create a new file called `new_layoffs.csv` with the information you just saw.

But pivot tables don't need to be performed just on the total number of rows. You can also apply a specified function
to be applied on a specific column. For instance, you could determine the total salary of the employees,
aggregated by whether or not they were fired and by what department they work for:

```bash
$ csvpivot sum layoffs.csv -r 3 -c 1 -v 2
,false,true
sales,85000,90000
engineering,175000,75000
```
So now we know that the combined salaries of all of the sales department employees who were fired is $90,000,
or the combination of the $25,000 salary and the $65,000 salary of the two sales department employees who were fired.

And there are a number of other functions you can apply on your given data.

### Supported functions
`csvpivot` has support for a number of functions, fitting into the following
categories: text functions, numeric functions, and type independent functions.

- The text functions don't require anything special of your text; they
will simply take the text data as it is. However, the data should be clean
before processing because the tool has no way of knowing that
BANK OF AMERICA and Bank of America NA are referring to the same thing.
- The numeric functions *require* that data be in a numeric format.
This includes the median, because the median will commonly be the mean
of two values. Note that there is no support currently for parsing thousands
separators or currency markers, as noted at the beginning of this user guide,
so you must remove them prior to using `csvpivot`. Additionally,
keep in mind that the standard deviation is referring to the *sample*
standard deviation, which is equivalent to 
$s = \sqrt{\frac{1}{N - 1}\sum_{i=1}^{N}{(x - \bar{x})^{2}}}$
rather than the population standard deviation, which is equivalent to
$\sigma = \sqrt{\frac{1}{N}\sum_{i=1}^{N}{(x - \mu)^{2}}}$
- type independent fields work on a number of different types and are
mainly designed for parsing numbers and dates. (In fact, `range` only works
on numbers and dates.) These all have default types that can be overridden
by using the `-i` or `-N` flags, which infer values as dates or parse them as numbers,
respectively.
    - `max` defaults to reading values as strings (mainly under the understanding
    that YYYYMMDD formatted dates sort in chronological order)
    - `min` works like `max` but for minimum values
    - `range` defaults to reading values as YYYYMMDD HH:MM:SS dates. That means
    that, unlike `max` and `min`, you can get an error on `range` even when
    using the `-i` flag. For dates, `range` returns the number of days in
    YYYYMMDD formatted fields and the number of second in YYYYMMDD HH:MM:SS
    formatted fields.

If your dates are formatted in YYYYMMDD HH:MM:SS formats, I recommend that you
don't have them parsed with the `-i` flag, because the `-i` flag has to read
through all of the data to determine the formatting of your dates.

Finally, you can use the `-e` flag to have `csvpivot` ignore empty fields. 
This uses the empty values from the [agate](https://agate.readthedocs.io)
library in Python. Specifically, it assumes that if you have any of the
following values, they are null values and should be ignored:
an empty string, 'n/a', 'none', 'null', 'nan'. The function determining whether
or not to lowercase values renders text into lowercase, so uppercase versions
of those phrases will be rendered as empty as well.

### Other Options
I've covered most of the available options already, but there are still
two that I haven't covered. Very quickly, there's here:

- Use the `-d` delimeter flag if your "comma-separated values" file is not
actually separated by commas. If you're using this on a tab-separated file,
write `-d '\t'`
- CSV files often use quote characters in fields, primarily to have the parser
ignore the delimeter within the quotes. (For instance, the
$75,000 donation in the Post's inaugural donations dataset I referenced earlier
uses quotes around "$75,000.00". This allows them to use a comma in the middle
of a cell within a CSV file. You can use the `-q` flag to override the default,
which assumes that double quotation marks like `"` serve as quote characters
in the document.

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