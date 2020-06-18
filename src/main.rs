#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches};
use once_cell::sync::Lazy;
use std::process;

use rust_decimal::Decimal;

use clipivot::aggfunc::*;
use clipivot::aggregation::{Aggregator, OutputOrder, ParsingStrategy};
use clipivot::cli_settings::CsvSettings;
use clipivot::errors::{CsvCliError, CsvCliResult};
use clipivot::parsing::{self, CustomDateObject, DecimalWrapper};

const ALLOWED_AGGFUNCS: [&str; 11] = [
    "count",
    "countunique",
    "max",
    "mean",
    "median",
    "min",
    "minmax",
    "mode",
    "range",
    "stddev",
    "sum",
];
static CLI_ARGS: Lazy<ArgMatches> = Lazy::new(|| {
    App::new("clipivot")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A tool for creating pivot tables from the command line.\n\
        For more information, visit https://www.github.com/maxblee/clipivot")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("aggfunc")
            .required(true)
            .index(1)
            .possible_values(&ALLOWED_AGGFUNCS)
            .help("The function you use to run across the pivot table.
            - count counts the number of matching records.
            - countunique counts the number of unique matching records.
            - max returns the maximum value of the records given a specified data type.
            - mean returns the mean.
            - median returns the median value. Requires numeric data.
            - min returns the minimum value of the records given a specified data type.
            - minmax returns both the minimum and maximum values of the records, split by a hyphen.
            - mode returns the most commonly appearing value.
            - range returns the difference between the minimum and maximum values. Returns the number of days in the case of dates.
            - stddev returns the sample standard deviation.
            - sum returns the sum of the values."))
        .arg(Arg::with_name("filename")
            .index(2)
            .help("The path to the file you want to create a pivot table from"))
        .arg(Arg::with_name("rows")
            .long("rows")
            .short("r")
            .takes_value(true)
            .multiple(true)
            .help("The name of the index(es) to aggregate on. Accepts string fieldnames or 0-indexed fields."))
        .arg(Arg::with_name("columns")
            .long("cols")
            .short("c")
            .takes_value(true)
            .multiple(true)
            .help("The name of the column(s) to aggregate on. Accepts string fieldnames or 0-indexed fields."))
        .arg(Arg::with_name("value")
            .long("val")
            .short("v")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("numeric")
            .short("N")
            .help("Parse values as numeric data. This is only necessary for min, max, and minmax, which can parse strings."))
        .arg(Arg::with_name("format")
            .short("F")
            .takes_value(true)
            .help("The format of a date field (e.g. %Y-%m-%d for dates like 2010-09-21)"))
        .arg(Arg::with_name("empty")
            .short("e")
            .help("Ignores empty/null values ('', NULL, NaN, NONE, NA, N/A)"))
        .arg(Arg::with_name("noheader")
            .long("no-header")
            .help("Skip the header row of the CSV file."))
        .arg(Arg::with_name("delim")
            .short("d")
            .long("delim")
            .takes_value(true)
            .help("The delimiter used to separate fields. Defaults to ','."))
        .arg(Arg::with_name("tab")
            .short("t")
            .help("Set the delimiter of the file to a tab."))
        .arg(Arg::with_name("indexcol")
            .short("I")
            .long("index-cols")
            .help("Display column names in index order. Defaults to sorted, ascending order."))
        .arg(Arg::with_name("desccol")
            .short("R")
            .long("desc-cols")
            .help("Display column names in sorted, descending order (default is ascending)"))
        .arg(Arg::with_name("ascrow")
            .short("A")
            .long("asc-rows")
            .help("Displays the rows in sorted, ascending order (default is index order)."))
        .arg(Arg::with_name("descrow")
            .short("D")
            .long("desc-rows")
            .help("Displays the rows in sorted, descending order (default is index order)."))
        .get_matches()
});
fn run_and_init<T, I, O>(
    arg_matches: &ArgMatches,
    parsing_strategy: ParsingStrategy,
) -> CsvCliResult<()>
where
    T: Accumulate<I, O>,
    I: std::str::FromStr,
    O: std::fmt::Display,
{
    // fn run_and_init<T: Accumulate<I,O>, I: std::str::FromStr, O: std::fmt::Display>(arg_matches: &ArgMatches) -> CsvCliResult<()> {
    let filename = arg_matches.value_of("filename");
    let delim_values = if arg_matches.is_present("tab") {
        Some(r"\t")
    } else {
        arg_matches.value_of("delim")
    };
    let settings =
        CsvSettings::parse_new(&filename, delim_values, !arg_matches.is_present("noheader"))?;
    if let Some(filepath) = filename {
        let rdr = settings.get_reader_from_path(filepath)?;
        agg_from_reader::<T, I, O, std::fs::File>(arg_matches, &settings, parsing_strategy, rdr)?;
    } else {
        let rdr = settings.get_reader_from_stdin();
        agg_from_reader::<T, I, O, std::io::Stdin>(arg_matches, &settings, parsing_strategy, rdr)?;
    }
    Ok(())
}

fn agg_from_reader<T, I, O, R>(
    arg_matches: &ArgMatches,
    settings: &CsvSettings,
    parsing_strategy: ParsingStrategy,
    mut reader: csv::Reader<R>,
) -> CsvCliResult<()>
where
    T: Accumulate<I, O>,
    I: std::str::FromStr,
    O: std::fmt::Display,
    R: std::io::Read,
{
    let headers = reader.headers()?;
    let mut agg = get_aggregator::<T, I, O>(
        arg_matches,
        &settings,
        parsing_strategy,
        &headers.iter().collect(),
    )?;
    agg.aggregate(reader)?;
    agg.write_results()?;
    Ok(())
}

fn get_aggregator<T, I, O>(
    arg_matches: &ArgMatches,
    settings: &CsvSettings,
    parsing_strategy: ParsingStrategy,
    headers: &Vec<&str>,
) -> CsvCliResult<Aggregator<T, I, O>>
where
    T: Accumulate<I, O>,
    I: std::str::FromStr,
    O: std::fmt::Display,
{
    let str_indexes = arg_matches
        .values_of("rows")
        .map_or(vec![], |v| v.collect());
    let index_cols = settings.get_field_indexes(&str_indexes, headers)?;
    let str_cols = arg_matches
        .values_of("columns")
        .map_or(vec![], |v| v.collect());
    let column_cols = settings.get_field_indexes(&str_cols, headers)?;
    let values_col = settings.get_field_index(arg_matches.value_of("value").unwrap(), headers)?;
    let skip_null = arg_matches.is_present("empty");
    let row_ordering_pair = (
        arg_matches.is_present("ascrow"),
        arg_matches.is_present("descrow"),
    );
    let row_order = match row_ordering_pair {
        (true, true) => Err(CsvCliError::InvalidConfiguration(
            "You can only enter one of the -A and -D flags".to_string(),
        )),
        (true, false) => Ok(OutputOrder::Ascending),
        (false, true) => Ok(OutputOrder::Descending),
        (false, false) => Ok(OutputOrder::IndexOrder),
    }?;
    let column_ordering_pair = (
        arg_matches.is_present("indexcol"),
        arg_matches.is_present("desccol"),
    );
    let column_order = match column_ordering_pair {
        (true, true) => Err(CsvCliError::InvalidConfiguration(
            "You can only enter one of the -I and -R flags".to_string(),
        )),
        (true, false) => Ok(OutputOrder::IndexOrder),
        (false, true) => Ok(OutputOrder::Descending),
        (false, false) => Ok(OutputOrder::Ascending),
    }?;
    let agg = Aggregator::new(
        index_cols,
        column_cols,
        values_col,
        skip_null,
        row_order,
        column_order,
        parsing_strategy,
    );
    Ok(agg)
}

pub fn run() -> CsvCliResult<()> {
    match CLI_ARGS.value_of("aggfunc").unwrap() {
        "count" => run_and_init::<Count<String>, String, usize>(&CLI_ARGS, ParsingStrategy::Text),
        "countunique" => {
            run_and_init::<CountUnique<String>, String, usize>(&CLI_ARGS, ParsingStrategy::Text)
        }
        "max" if (CLI_ARGS.is_present("numeric") && CLI_ARGS.is_present("format")) => {
            Err(CsvCliError::InvalidConfiguration(
                "You can only enter one of the -N and -F flags/options".to_string(),
            ))
        }
        "max" if CLI_ARGS.is_present("numeric") => {
            run_and_init::<Maximum<f64>, f64, f64>(&CLI_ARGS, ParsingStrategy::Numeric)
        }
        "max" if CLI_ARGS.is_present("format") => run_and_init::<
            Maximum<CustomDateObject>,
            CustomDateObject,
            CustomDateObject,
        >(&CLI_ARGS, ParsingStrategy::Date),
        "max" => run_and_init::<Maximum<String>, String, String>(&CLI_ARGS, ParsingStrategy::Text),
        "mean" => run_and_init::<Mean, DecimalWrapper, DecimalWrapper>(
            &CLI_ARGS,
            ParsingStrategy::Numeric,
        ),
        "median" => run_and_init::<Median, DecimalWrapper, DecimalWrapper>(
            &CLI_ARGS,
            ParsingStrategy::Numeric,
        ),
        "min" if (CLI_ARGS.is_present("numeric") && CLI_ARGS.is_present("format")) => {
            Err(CsvCliError::InvalidConfiguration(
                "You can only enter one of the -N and -F flags/options".to_string(),
            ))
        }
        "min" if CLI_ARGS.is_present("numeric") => {
            run_and_init::<Minimum<f64>, f64, f64>(&CLI_ARGS, ParsingStrategy::Numeric)
        }
        "min" if CLI_ARGS.is_present("format") => run_and_init::<
            Minimum<CustomDateObject>,
            CustomDateObject,
            CustomDateObject,
        >(&CLI_ARGS, ParsingStrategy::Date),
        "min" => run_and_init::<Minimum<String>, String, String>(&CLI_ARGS, ParsingStrategy::Text),
        "minmax" if (CLI_ARGS.is_present("numeric") && CLI_ARGS.is_present("format")) => {
            Err(CsvCliError::InvalidConfiguration(
                "You can only enter one of the -N and -F flags/options".to_string(),
            ))
        }
        "minmax" if CLI_ARGS.is_present("numeric") => {
            run_and_init::<MinMax<f64>, f64, String>(&CLI_ARGS, ParsingStrategy::Numeric)
        }
        "minmax" if CLI_ARGS.is_present("format") => run_and_init::<
            MinMax<CustomDateObject>,
            CustomDateObject,
            String,
        >(&CLI_ARGS, ParsingStrategy::Date),
        "minmax" => {
            run_and_init::<MinMax<String>, String, String>(&CLI_ARGS, ParsingStrategy::Text)
        }
        "range" if CLI_ARGS.is_present("format") => run_and_init::<
            Range<CustomDateObject, f64>,
            CustomDateObject,
            f64,
        >(&CLI_ARGS, ParsingStrategy::Date),
        "range" => run_and_init::<Range<DecimalWrapper, Decimal>, DecimalWrapper, Decimal>(
            &CLI_ARGS,
            ParsingStrategy::Numeric,
        ),
        "stddev" => run_and_init::<StdDev, f64, f64>(&CLI_ARGS, ParsingStrategy::Numeric),
        "sum" => run_and_init::<Sum<DecimalWrapper>, DecimalWrapper, DecimalWrapper>(
            &CLI_ARGS,
            ParsingStrategy::Numeric,
        ),
        _ => unreachable!(),
    }
}

fn main() {
    if let Some(date_format) = CLI_ARGS.value_of("format") {
        parsing::set_date_format(date_format);
    }

    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
