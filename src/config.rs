use Clap::ArgMatches;
use errors::CsvPivotError;


fn parse_delimiter(filename: &Option<&str>, arg_matches: &ArgMatches) -> Result<u8, CsvPivotError> {
    let default_delim = match filename {
        _ if arg_matches.is_present("tab") => vec![b'\t'],
        _ if arg_matches.is_present("delim") => {
            let delim = arg_matches.value_of("delim").unwrap();
            if let r"\t" = delim {
                vec[b'\t']
            } else { delim.as_bytes().to_vec() }
        },
        // altered from https://github.com/BurntSushi/xsv/blob/master/src/config.rs
        Some(fname) if fname.ends_with(".tsv") || fname.ends_with(".tab") => vec![b'\t'],
        _ => vec![b'\t']
    };
    if !(default_delim.len() == 1) {
        let msg = format!(
            "Could not convert `{}` delimiter to a single ASCII character",
             String::from_utf8(default_delim).unwrap()
             );
        return Err(CsvPivotError::InvalidConfiguration(msg));
    }
    Ok(default_delim)
}

/// This struct is intended for converting from Clap's `ArgMatches` to the `Aggregator` struct
#[derive(Debug, PartialEq)]
pub struct CliConfig<U>
where
    U: AggregationMethod,
{
    // set as an option so I can handle standard input
    filename: Option<String>,
    aggregator: Aggregator<U>,
    has_header: bool,
    delimiter: u8,
    values_col: String,
    column_cols: Vec<String>,
    indexes: Vec<String>,
}

impl<U: AggregationMethod> CliConfig<U> {
    /// Creates a new, basic CliConfig
    pub fn new() -> CliConfig<U>  {
        CliConfig {
            filename: None,
            aggregator: Aggregator::new(),
            has_header: true,
            delimiter: b',',
            values_col: "".to_string(),
            column_cols: vec![],
            indexes: vec![],
        }
    }
    /// Takes argument matches from main and tries to convert them into CliConfig
    pub fn from_arg_matches(arg_matches: ArgMatches) -> Result<CliConfig<U>, CsvPivotError> {
        let base_config: CliConfig<U> = CliConfig::new();
        let values_col = arg_matches.value_of("value").unwrap().to_string();    // unwrap safe because required arg
        let column_cols = arg_matches.values_of("rows")
            .map_or(vec![], |it| it.map(|val| val.to_string()).collect());
        let indexes = arg_matches.values_of("columns")
            .map_or(vec![], |it| it.map(|val| val.to_string()).collect());
        let filename = arg_matches.value_of("filename").map(String::from);
        // TODO This is hacky
        let parser = base_config.get_parser(&arg_matches);
        let aggregator: Aggregator<U> = Aggregator::from_parser(parser);

        let delimiter = parse_delimiter(&filename, &arg_matches)?;

        let cfg = CliConfig {
            filename,
            aggregator,
            has_header: !arg_matches.is_present("noheader"),
            delimiter: delimiter[0],
            values_col,
            indexes,
            column_cols
        };
        Ok(cfg)
    }
    fn get_parsing_approach(&self, parse_numeric: bool, parse_date: bool) -> ParsingType {
        match U::get_aggtype() {
            AggTypes::Count => ParsingType::Text(None),
            AggTypes::CountUnique => ParsingType::Text(None),
            AggTypes::Mode => ParsingType::Text(None),
            AggTypes::Mean => ParsingType::Numeric(None),
            AggTypes::Median => ParsingType::Numeric(None),
            AggTypes::Sum => ParsingType::Numeric(None),
            AggTypes::StdDev => ParsingType::FloatingPoint(None),
            AggTypes::Minimum if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Maximum if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Range if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Maximum if parse_date => ParsingType::DateTypes(None),
            AggTypes::Minimum if parse_date => ParsingType::DateTypes(None),
            AggTypes::Range => ParsingType::DateTypes(None),
            AggTypes::Minimum => ParsingType::Text(None),
            AggTypes::Maximum => ParsingType::Text(None)
        }
    }

    fn get_parser(&self, arg_matches: &ArgMatches) -> ParsingHelper {
        let parse_numeric = arg_matches.is_present("numeric");
        let infer_date = arg_matches.is_present("infer");
        let parse_type = self.get_parsing_approach(parse_numeric, infer_date);
        ParsingHelper::from_parsing_type(parse_type)
            .parse_empty_vals(!arg_matches.is_present("empty"))
    }
    /// Converts from a file path to either a CSV reader or a CSV error.
    ///
    /// In the spirit of DRY, it would be nice to avoid replicating code from this and
    /// `get_reader_from_stdin`.
    ///
    /// This should be able to be done simply by creating a function
    /// that returns a `csv::ReaderBuilder` and then applying that to both functions.
    /// That will become especially important when I eventually get around to adding
    /// additional features, like allowing users to select a delimeter other than ','.
    // TODO: Refactor this code
    pub fn get_reader_from_path(&self) -> Result<csv::Reader<fs::File>, csv::Error> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            // this function is only run if self.filename.is_some() so unwrap() is fine
            .from_path(self.filename.as_ref().unwrap())
    }

    /// Converts from standard input to a CSV reader.
    pub fn get_reader_from_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_reader(io::stdin())
    }

    fn get_header_idx(&self, colname: &str, headers: &Vec<&str>) -> Result<usize, CsvPivotError> {
        let mut in_quotes = false;
        let mut order_specification = false; // True if we've passed a '['
        // fieldname occurrence is the order in which a field occurs. Defaults to 0.
        let mut fieldname_occurrence : String = "".to_string(); 
        let mut occurrence_start = 0;
        let mut occurrence_end = 0;
        let header_length = headers.len();  
        let mut all_numeric = true; // default to reading the field as a 0-indexed number
        let chars = colname.chars();
        if (self.has_header) {
            for (i, c) in chars.enumerate() {
                if !(c.is_ascii_digit()) { all_numeric = false; }
                if (c == '\'' || c == '\"') && !(in_quotes) { 
                    in_quotes = true; 
                    continue;
                    }
                if (c == '\'' || c == '\"') && in_quotes { 
                    in_quotes = false; 
                    continue;
                    }
                if in_quotes { 
                    continue; 
                    }
                if c == '[' && !(in_quotes) { 
                    order_specification = true; 
                    if i + 1 < colname.len() { occurrence_start = i + 1; }
                    continue;
                }
                if c == ']' { 
                    occurrence_end = i;
                    continue; 
                    }
                if order_specification {
                    if !(c.is_ascii_digit()) { 
                        let msg = format!(
                            "Could not parse the fieldname {}. You may need to encapsulate the field in quotes",
                            colname
                        );
                        return Err(CsvPivotError::InvalidConfiguration(msg));
                    }
                    fieldname_occurrence.push_str(&c.to_string());
                }
            }
        }
        if all_numeric {
            let parsed_val : usize = colname.parse()?;
            if !((0 <= parsed_val) && (parsed_val < header_length)) {
                println!("{}", headers);
                let msg = format!("Column selection must be between
                0 <= selection < {}", header_length);
                return Err(CsvPivotError::InvalidConfiguration(msg));
            } else { return Ok(parsed_val); }
        } else if order_specification {
            let orig_end = match occurrence_start {
                i if i - 1 >= 0 => Ok(i - 1),
                i => Err(CsvPivotError::InvalidConfiguration("Couldn't parse field.".to_string()))
            }?;
            let orig_name = &colname[..orig_end];
            let parsed_val : usize = colname[occurrence_start..occurrence_end].parse()?;
            let mut count = 0;
            for (i, field) in headers.iter().enumerate() {
                if field == &orig_name {
                    count += 1;
                    if count == parsed_val + 1 { {
                        return Ok(i); 
                        }}
                }
            }
            let msg = format!("There are only {} occurrences of the fieldname {}", count, orig_name);
            return Err(CsvPivotError::InvalidConfiguration(msg));
        } else { match headers.iter().position(|&i| i == colname) {
            Some(position) => { return Ok(position); },
            None => {
                let msg = format!("Could not find the fieldname `{}` in the header", colname);
                return Err(CsvPivotError::InvalidConfiguration(msg));
            }
        }
        }
    }

    fn parse_combined_col(&self, combined_name: &str) -> Result<Vec<String>, CsvPivotError> {
        let mut output_vec = Vec::new();
        let mut in_quotes = false;
        let mut cur_string = String::new();
        // Option<char> where None == No previous quote, Some('\'' == previous single quote)
        let mut prev_quote = None;
        let mut last_parsed = None;
        for c in combined_name.chars() {
            last_parsed = Some(c);
            if !in_quotes {
                if (c == '\'' || c == '\"') && cur_string.is_empty() {
                    prev_quote = Some(c);
                    in_quotes = true;
                    cur_string.push(c);
                    continue;
                } if c == ',' {
                    output_vec.push(cur_string);
                    cur_string = String::new();
                } else {
                    cur_string.push(c);
                }
            } else {
                if c== '\'' || c== '\"' {
                    if prev_quote != Some(c) {
                        return Err(CsvPivotError::InvalidConfiguration("Quotes inside fieldname were not properly closed".to_string()))
                    }
                    in_quotes = false;
                    cur_string.push(c);
                    continue;
                } else { 
                    cur_string.push(c); 
                    }
            }
        }
        if !cur_string.is_empty() {
            output_vec.push(cur_string);
        }
        if in_quotes { 
            return Err(CsvPivotError::InvalidConfiguration("Quotes inside fieldname were not properly closed".to_string()));
            }
        if last_parsed == Some(',') {
            return Err(CsvPivotError::InvalidConfiguration("One of the fieldnames ends with an unquoted comma".to_string()));
        }
        Ok(output_vec)
    }

    fn get_multiple_header_columns(&self, colnames: &Vec<String>) -> Result<Vec<String>, CsvPivotError> {
        let mut expected_columns = Vec::new();
        for col in colnames {
            let parsed_col = self.parse_combined_col(&col)?;
            expected_columns.extend(parsed_col);
        }
        Ok(expected_columns)
    }

    fn get_idx_vec(&self, expected_cols: &Vec<String>, headers: &Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
        let mut all_cols = Vec::new();
        for col in expected_cols {
            let col_idx = self.get_header_idx(&col, headers)?;
            all_cols.push(col_idx);
        }
        let mut parsed_cols = HashSet::new();
        let mut output_cols = Vec::new();
        for col in all_cols {
            if !parsed_cols.contains(&col) {
                output_cols.push(col);
            } else {
                parsed_cols.insert(col);
            }
        }
        Ok(output_cols)
    }

    fn validate_columns(&mut self, headers: &Vec<&str>) -> Result<(), CsvPivotError> {
        // validates the aggregation columns and then updates the aggregator
        let expected_indexes = self.get_multiple_header_columns(&self.indexes)?;
        let index_vec = self.get_idx_vec(&expected_indexes, headers)?;
        let expected_columns = self.get_multiple_header_columns(&self.column_cols)?;
        let column_vec = self.get_idx_vec(&expected_columns, headers)?;
        let values_vec = self.get_header_idx(&self.values_col, headers)?;
        // need self.aggregator = .. right now bc set_indexes etc return Self (rather than mutating)
        // TODO clean up method chaining to avoid this mess
        self.aggregator
            .set_indexes(index_vec)
            .set_columns(column_vec)
            .set_value_column(values_vec);
        Ok(())
    }

    /// Runs the `Aggregator` for the given type.
    pub fn run_config(&mut self) -> Result<(), CsvPivotError> {
        if self.filename.is_some() {
            let mut rdr = self.get_reader_from_path()?;
            let headers = rdr.headers()?;
            self.validate_columns(&headers.iter().collect())?;
            self.aggregator.aggregate_from_file(rdr)?;
        } else {
            let mut rdr = self.get_reader_from_stdin();
            let headers = rdr.headers()?;
            self.validate_columns(&headers.iter().collect())?;
            self.aggregator.aggregate_from_stdin(rdr)?;
        }
        self.aggregator.write_results()?;
        Ok(())
    }
}
