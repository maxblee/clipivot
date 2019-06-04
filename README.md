# csvpivot
A tool for creating pivot tables from the command line.

## The pitch
Pivot tables are a useful tool for quickly exploring data. You can use them to identify potential stories and to identify potential
data quality and data consistency problems. Over the course of this tutorial, I'll go over some specific examples that show how you
can use this tool, and how you can use pivot tables in general, for all of these purposes. 

But before I do that, I want to talk about why I designed `csvpivot.` 

There are two things I hope this command-line interface does well. First, I hope the tool makes it easier to set up pivot tables for simple
data exploration tasks involving large datasets. I want it to serve as a way to allow people to interview data before loading it into SQL
and before worrying about memory constrains in Python or R. I think the tool can help people figure out what sorts of things they might need
to do to clean a given dataset, and what sort of analysis they want to perform on the dataset. It should also be useful for helping people
figure out which parts of a dataset they can filter out so they can read it into memory in Python or R.

Second, I want it to be as easy as possible for people to log queries using this tool. One of the weaknesses of command-line interfaces in
comparison to programming and query languages is that documenting work in CLIs typically involves copying and pasting queries. By comparison,
the `--log` parameter in `csvpivot` makes it easy to keep a record of your work. More on that later.

## Logging
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
