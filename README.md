# csvpivot
A tool for creating pivot tables from the command line.

## Table of Contents
* **[What is this?](#what-is-this-the-pitch)**
* **[What isn't this?](#what-isnt-this-the-anti-pitch)**
* **[Why pivot tables?](#why-pivot-tables)**
* **[Usage Guide](#usage-guide)**
    - **[Installation](#installation)**
    - **[Basic Usage](#basic-usage)**
    - **[Advanced Usage](#advanced-usage)**
        - **[Additional Options and Flags](#additional-options-and-flags)**
        - **[Using csvpivot with other tools](#using-csvpivot-with-other-tools)**
* **[Performance Benchmarks](#performance-benchmarks)**
* **[Developer Guide](#developer-guide)**
* **[Contact Me](#contact-me)**

## What is this? (The pitch)
Pivot tables are a useful tool for quickly exploring data. As I go through this tutorial, you'll see some ways that
you can use pivot tables to identify potential stories and to identify potential data quality problems.

But for large datasets, they can be tricky to set up.

`csvpivot` is meant to address this. It is fairly easy to use for small datasets and large datasets alike, and it
aggregates records one row at a time, so you're unlikely to run into memory issues, as you might using a library
like Python's `pandas`.

## What isn't this? (The anti-pitch)
* `csvpivot` is a tool, not a toolkit. There are too many good CSV toolkits out there for me to be able to justify
creating a new one. However, `csvpivot` *is* designed to play nicely with other command-line CSV toolkits. If you
go to [the end of the usage guide](#using-csvpivot-with-other-tools), I'll show you some of my favorite toolkits and how you can use `csvpivot` in
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

## Why pivot tables?
Before I go into detail about how to use `csvpivot`, I want to show you the kinds of things pivot tables can be useful
for. We'll use `csvpivot`, so you'll get a sense of the syntax, but this section isn't designed to teach you how
to use the tool. It's designed to explain why you'd want to use it or anything like it. (If you already know what
a pivot table is and just want to see how to use this tool, feel free to skip to the [usage guide](#usage-guide) or
type `csvpivot --help` after installation.)

We're going to start by looking at donation data for President Trump's 2017 Inauguration. *The Washington Post*
published [the data](https://github.com/washingtonpost/data-inaugural-committee) we're going to use and wrote
[the story](https://www.washingtonpost.com/politics/2018/12/14/how-money-flowed-into-out-trumps-inaugural-committee/?utm_term=.54dbde88e0a7)
that forms the basis for this part of our analysis. Specifically, here's the part of the story we'll be looking at:

> Of the money that was raised, more than half came from individual donors, according to data compiled by CBNC’s 
Christina Wilkie and the Center for Responsive Politics. No single entity gave more than casino mogul Sheldon Adelson,
 who ponied up $5 million.
 
How do we validate these findings? Well, if we look at our data, there's a field called `entity_type` that refers to
the type of donation (whether it came from an individual donor or an organization, primarily). Each row of the file,
meanwhile, refers to an individual transaction. So in order to find out what types of entities donated the most
money to the inaugural committee, we need to compare the sum of all of the donations from individuals to the sum
of all the donations from organizations.

That's what pivot tables are meant for. In a pivot table, each row corresponds to a unique value along a field or
a concatenation of fields, and each column corresponds to a unique value along a (typically different) field or
concatenation of fields. With each cell, we apply a function (like a summation) across all values in a value column
you set. So in our case, we want to set the rows of our pivot table to the `entity_type` field, set our function
to `sum`, and set our values column to `donation` in order to figure out the total amount of money donated by `entity_type`.

Here's what that looks like in `csvpivot`:

```csv
csvtk replace -p '(\$|,)' -r '' -f 5 inaugural_donations.csv | csvpivot sum -r 2 -v 4 | xsv sort -N -s total -R
```
That first replaces all dollar signs and commas in the `donations` field with empty strings so `csvpivot` can parse
them as numbers. Then, `csvpivot` computes the summation of all of the donations by `entity_type`. And finally,
we sort the total values in reverse order. (I'll talk more about these other toolkits in 
[Using csvpivot with other tools](#using-csvpivot-with-other-tools).)

When we do that, this is our output: (NOTE: THIS DOESN'T WORK YET. WILL SET UP TOMORROW)

```csv
,total
IND,59538730.00
ORG,47171478.29
MOC,5100.00
```
Sure enough! As the article states, the majority of donations came from individual donors. Now, let's see how much
money Sheldon Adelson gave:

```csv
$ csvtk replace -p '(\$|,)' -r '' -f 5 inaugural_donations.csv | csvpivot sum -r 3 -v 4 | xsv sort -N -s total -R | xsv slice -e 2
,total
"ADELSON, SHELDON, G",5000000
AT&T,2082483.43
```
And again, we've verified our findings, showing us that no single entity, including organizations like AT&T, gave more
to the inaugural committee than Sheldon Adelson.

TODO

## Usage Guide
### Installation
TODO
### Basic Usage
The best place to start for a general understanding of `csvpivot` is

```
csvpivot --help
This shows the output of help
```
But I'll also provide a quick tour of the program. 

Say you have a sample CSV file called `layoffs.csv` of people recently fired from a company that looks like this:
```csv
id,was_fired,salary,department
1,true,25000,sales
2,true,75000,engineering
3,false,175000,engineering
4,true,65000,sales
5,false,85000,sales
```
Now, you want to figure out the number of people who were fired from the sales department. Enter pivot tables.

In order to figure out how many people were fired from the sales department, you need to count the number of rows
where `was_fired` is `true` *and* where `department` is sales. Here's how you do that using `csvpivot`:
```bash
csvpivot count layoffs.csv --rows=3 --cols=1 --val=0
```
That will print out a new CSV that looks like
```csv
,true,false
sales,2,1
engineering,1,1 
```

### Advanced Usage
The following guide provides some information about more advanced usage. I believe that most of the flags and options are adequately covered in `csvpivot --help`. But there are a few flags, like `--sort`, that are a little more complicated.
#### Additional Options and Flags
TODO
#### Using csvpivot with other tools
TODO

## Performance Benchmarks
TODO

## Developer Guide
TODO

## Contact Me
If you have any questions about `csvpivot` or if you have identified any bugs in the program or you want
to contribute to it, please send me an email at maxbmhlee@gmail.com or contact me through Twitter. 
I'm [@maxblee](https://twitter.com/maxblee). And if you wind up using `csvpivot` in any of your projects,
I'd love to know.
