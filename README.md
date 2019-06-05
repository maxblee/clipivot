# csvpivot
A tool for creating pivot tables from the command line.

## Table of Contents
* **[Why should you use this?](#why-should-you-use-this-the-pitch)**
* **[Why shouldn't you use it?](#why-shouldnt-you-use-it-the-anti-pitch)**
* **[Why pivot tables?](#why-pivot-tables)**
* **[Usage Guide](#usage-guide)**
    - **[Installation](#installation)**
    - **[Basic Usage](#basic-usage)**
    - **[Logging](#logging)**
    - **[Advanced Usage](#advanced-usage)**
        - **[Additional Options and Flags](#additional-options-and-flags)**
        - **[Piping](#piping)**
* **[Performance Benchmarks](#performance-benchmarks)**
* **[Developer Guide](#developer-guide)**
* **[Contact Me](#contact-me)**

## Why should you use this? (The pitch)
Pivot tables are a useful tool for quickly exploring data. As I go through this tutorial, you'll see some ways that you can use pivot tables to identify potential stories and to identify data quality and data consistency problems. 

But in order to set up a pivot table on a large dataset (as in larger than about ~1 GB, depending on how much RAM you have), you need
to either load the data into SQL and use `GROUP BY` queries, or you need to create pivot tables on small-ish chunks of data using libraries like `pandas` and use concatenation methods to consolidate the rows and columns of different pivot tables into a single
pivot table. The first method isn't ideal for data exploration tasks; the latter method is a pain and can be prone to error.

This library is meant to address that problem. Written in Rust, `csvpivot` is designed to work quickly on large datasets. And
I've written all of the aggregation methods (e.g. sum, mean, etc.) so they work on chunks as well as on entire datasets. It still works
on small datasets, of course, so it can be useful if you find it more convenient to work from the command line. But it's particularly
useful for handling multi-gigabyte files, where there isn't necessarily a great alternative for simple data exploration tasks.

And one last thing. As a journalist, I've written this tool with an eye for keeping a record of queries you've written, so colleagues
can replicate your work and so you can remember what queries led to interesting findings. That's not a reason to use
this command-line interface in itself, but it is a handy (and slightly more sophisticated) alternative to
```
csvpivot sample.csv count -r=name -c=type > sample_pivot.csv && 
> echo "csvpivot sample.csv count -r=name -c=type > sample_pivot.csv" >> data-diary.txt
```
## Why shouldn't you use it? (The anti-pitch)
There are two main places where it doesn't make sense to use `csvpivot`.

First, I don't expect that `csvpivot` will run any faster than SQL, so if you've already loaded data into SQL, I don't think it particularly makes sense to use this tool.

Second, `csvpivot` does not have the flexibility of methods like `pandas.pivot_table`. Currently, `csvpivot` supports count, sum, mean, median, min, max, and stdev (standard deviation). If there is a function you need to calculate outside of that list, you'll probably want a different tool. (This differs from pandas, for instance, where you can use the `aggfunc` parameter to apply any function to the aggregation.) Alternatively, you can contact me if you have a function that you anticipate other people might use, or you can modify the existing code to add functionality. (This uses the MIT license, so you don't have to worry about copyright/copyleft issues.)
## Why pivot tables?
Before I write about how to use `csvpivot`, I want to show you what's so powerful about pivot tables in the first place to help you figure out how to use them and to help you figure out why `csvpivot` might be useful.

TODO
## Usage Guide
### Installation
TODO
### Basic Usage
The best place to start for a general understanding of `csvpivot` is probably

```
csvpivot --help
This shows the output of help
```
But I'll also provide a general description of how the program is used.

There are two main positional parameters you need to use, in addition to the name of the executable (i.e. `csvpivot`):
the filename and the function. The filename can be the relative or absolute path to the file, but URLs are not currently supported. The
function, meanwhile, can be one of the following: count, sum, mean, median, min, max, and stdev (standard deviation).

In addition, there are three main options you'll wind up using: `-r` or `--rows`, `-c` or `--cols`, and `-v` or `--vals`. Of these, only `--vals` is required. If you do not include `--rows` or `--cols`, the program will automatically compute totals.

As an example, let's take a CSV file called `layoffs.csv` that looks like this:
```csv
id,was_fired,salary,department
1,true,25000,sales
2,true,75000,engineering
3,false,175000,engineering
4,false,65000,sales
5,false,85000,sales
```
Say, we just want to get the total salary of all of the people who were just fired. To do that, we can simply type
```
csvpivot layoffs.csv sum --rows=was_fired --vals=salary
```
And we'll get a pivot table that looks like this:
```csv
was_fired,salary.total
false,325000
true,100000
```
Or we can make the values from `was_fired` our column names. (Note: As a general practice, you should avoid doing this; there should
generally be more row names in your generated pivot tables than there are column names. However, it's fairly reasonable to use boolean
fields as column names.)
```
csvpivot layoffs.csv sum --cols=was_fired --vals=salary
```
Which will produce
```csv
totals,was_fired.false,was_fired.true
salary.total,325000,100000
```
Now, for a more typical query. Say we want to figure out the total salary of the people who were just fired, segmented by the department they work in. Again, that's fairly simple:
```
csvpivot layoffs.csv sum --cols=was_fired --rows=department --vals=salary
```
Which will output
```csv
department,was_fired.false,was_fired.true
engineering,175000,75000
sales,150000,25000
```
Finally, `--cols` and `--rows` both support multiple values. To show you what this does, let's look at a sample from the excellent [stolen guns database](https://www.thetrace.org/missing-pieces/) from The Trace. (I randomly selected this sample using the command-line CSV exploring tool [xsv](https://github.com/BurntSushi/xsv). We'll call this file `sample_gun_data.csv`.
```csv
RECORD ID,STATE,LOCATION,STATUS,AGENCY,AGENCY STATUS
241042,OR,PORTLAND,RECOVERED,PORTLAND PD,RECOVERED GUNS
363989,WA,SNOHOMISH CO,STOLEN,SNOHOMISH CO SO,STOLEN
652347,TX,WACO,STOLEN,WACO PD,STOLEN
662786,WA,KING CO,STOLEN,KING CO SO,STOLEN
663444,WA,KING CO,RECOVERED,KING CO SO,SAFEKEEPING
78167,CO,DENVER,LOST/STOLEN,DENVER PD,LOST/STOLEN
160381,MD,BALTIMORE,RECOVERED,BALTIMORE PD,EVIDENCE
393455,CA,KERN CO,STOLEN,KERN CO SO,STOLEN
325529,KS,WICHITA,RECOVERED,WICHITA PD,CRIME GUN
544707,GA,SAVANNAH,RECOVERED,SAVANNAH PD,SAFEKEEPING
```
Because we're only dealing with 10 records, we're obviously not going to find anything interesting here. But as an exercise, let's try to figure out how many firearms were stolen by location in this data. As you can probably tell, simply running
```
csvpivot sample_gun_data.csv count --rows=LOCATION --cols=STATUS --vals="RECORD ID"
```
isn't going to work across the whole dataset, because it will assume that places like COLUMBUS, GA and COLUMBUS,OH are the same (since they would both have the value of COLUMBUS in the LOCATION column). Instead, we'll need to concatenate the rows LOCATION and STATE:
```
csvpivot sample_gun_data.csv count --rows=STATE,LOCATION --cols=STATUS --vals="RECORD ID"
```
That will output:
```csv
STATE.LOCATION,STATUS.LOST/STOLEN,STATUS.RECOVERED,STATUS.RECOVERED(STOLEN),STATUS.STOLEN,STATUS.STOLEN (RECOVERED)
AZ.GLENDALE,0,0,0,1,0
FL.GAINESVILLE,0,0,1,0,0
FL.LEE CO,0,0,1,0,0
MD.BALTIMORE,0,1,0,0,0
TX.ARLINGTON,0,0,0,0,1
TX.AUSTIN,1,0,0,0,0
TX.DALLAS,0,2,0,0,0
TX.SAN ANTONIO,0,0,1,0,0
WA.SEATTLE,0,1,0,0,0
```
Notice the order of the rows. That will determine the ultimate formatting of the records, as well as how the records are sorted. For example, had you set `--rows=LOCATION,STATE` the last row would be `SEATTLE.WA,0,1,0,0,0`.

You can also give the `--cols` option multiple values. Just like `--rows`, the records will appear as `Field1.Field2.Field3`. You cannot, however, set multiple values fields, nor can you set multiple function arguments. The reason for this is both opinionated and practical. Because this tool is built for data exploration, I believe it is best used for single function, single column operations. And this tool works with piping and standard input, explained later in this tutorial, so there are additional ways you can replicate this behavior if you're certain you want to.
### Logging
Say you found something interesting using a query in `csvpivot` and you want to keep a record of it. You could copy and paste that query
into a data diary, after writing down the date and time you wrote the query. Or you could use the `--log` parameter in `csvpivot.`

If you add the `--log` parameter to a query, `csvpivot` will automatically look for a file called `data-diary.txt` in your current directory.
If that file does not exist, it will automatically create the file.

Alternatively, `--log` takes an optional `filename` parameter, allowing you to use your own data diary. It will not write over any of your
data; instead, if the file exists, it will simply add a line break and begin a new record.

By default, `--log` will run the `csvpivot` command. But you can add a `--no-run` parameter to log the query without running the program.

As an example, if you type
```
csvpivot sample.csv count -r=name -c=type --log notes.txt --no-run > sample_pivot.csv
```
at 10:00 a.m. local time on August 3, 2019, `csvpivot` will prompt you to provide some information about what your query shows:
```
Describe what this query does, why you ran it, and what it shows:
Ran counts on name field to see if there are any oddly frequent names. Found 33,475 records were empty.
```
Then, if you do not have an existing file called `notes.txt`, it will create a `notes.txt` file that looks like:
```
Data diary notes.txt was created 2019-08-03 10:00:00
2019-08-03 10:00:00
    Ran counts on name field to see if there are any oddly frequent names. Found 33,475 records were empty.
    Query:  csvpivot mydata.csv count -i=name
```
If the `notes.txt` file already existed, `csvpivot` will simply add a line break and add the last three lines to your file.

### Advanced Usage
The following guide provides some information about more advanced usage. I believe that most of the flags and options are adequately covered in `csvpivot --help`. But there are a few flags, like `--sort`, that are a little more complicated.
#### Additional Options and Flags
TODO
#### Piping
TODO

## Performance Benchmarks
TODO

## Developer Guide
TODO

## Contact Me
If you have any questions about `csvpivot` or if you have identified any bugs in the program or you want to contribute to it, please send me an email at maxbmhlee@gmail.com or contact me through Twitter. I'm [@maxblee](https://twitter.com/maxblee). And if you wind up using `csvpivot` in any of your projects, I'd love to know.
