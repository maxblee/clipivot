# csvpivot
A tool for creating pivot tables from the command line.

## Table of Contents
* **[Why should you use this?](#why-should-you-use-this-the-pitch)**
* **[Why shouldn't you use it?](#why-shouldnt-you-use-it-the-anti-pitch)**
* **[Usage Guide](#usage-guide)**
    - **[Logging](#logging)**

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
csvpivot sample.csv count -r=name -c=type > sample_pivot.csv && echo csvpivot sample.csv count -r=name -c=type > sample_pivot.csv
```
## Why shouldn't you use it? (The anti-pitch)
There are two main places where it doesn't make sense to use `csvpivot`.

First, I don't expect that `csvpivot` will run any faster than SQL, so if you've already loaded data into SQL, I don't think it particularly makes sense to use this tool.

Second `csvpivot` does not have the flexibility of methods like `pandas.pivot_table`, so if you have a use case where you need to calculate values over a function not supported in `csvpivot`, there isn't a great way to do that. (Pandas allows you to use the `aggfunc` argument to use any function for the values of the cells; `csvpivot` does not.) That said, if you have a function that you
anticipate other people might use, please contact me and I can incorporate it into the tool.

## Usage Guide
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
csvpivot mydata.csv count -i=name --log notes.txt --no-run
```
at 10:00 a.m. local time on August 3, 2019, `csvpivot` will prompt you to provide some information about what your query shows:
```
Describe what this query does, why you ran it, and what it shows:
  Ran counts on name field to see if there are any oddly frequent names. Found 33,475 records were empty.
```
Then, if you do not have an existing file called `notes.txt`, it will create a `notes.txt` file that looks like:
```
Data diary analyzing mydata.csv. Created 2019-08-03 10:00:00

2019-08-03 10:00:00
  Ran counts on name field to see if there are any oddly frequent names. Found 33,475 records were empty.
  Query:  csvpivot mydata.csv count -i=name
```
If the `notes.txt` file already existed, `csvpivot` will simply add a line break and add the last three lines to your file.
