use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use ReceiptParseError::*;

pub struct Item {
    pub name: String,
    pub consumer: String,
    // In cents / pence
    pub single_price: u32,
    pub count: u32,
}

impl Item {
    pub fn total_price(&self) -> u32 {
        self.single_price * self.count
    }

    /// # Examples of ways an item can be defined in the file
    /// ```
    /// use std::path::PathBuf;
    /// use receipt_processor::receipt::Item;
    /// let file = PathBuf::new();
    ///
    /// let result = Item::parse(&file, "15 chocolate donut g").unwrap();
    /// assert_eq!(result.name, String::from("chocolate donut"));
    /// assert_eq!(result.consumer, String::from("g"));
    /// assert_eq!(result.single_price, 1500);
    /// assert_eq!(result.count, 1);
    ///
    /// let result = Item::parse(&file, "0.3 x4 pizza m").unwrap();
    /// assert_eq!(result.name, String::from("pizza"));
    /// assert_eq!(result.consumer, String::from("m"));
    /// assert_eq!(result.single_price, 30);
    /// assert_eq!(result.count, 4);
    ///
    /// let result = Item::parse(&file, "2 p");
    /// assert!(result.is_err());
    ///
    /// let result = Item::parse(&file, "2 x3 k");
    /// assert!(result.is_err());
    /// ```
    pub fn parse(file_path: &PathBuf, line: &str) -> Result<Item, ReceiptParseError> {
        let split: Vec<&str> = line.trim().split(" ").collect();
        if split.len() < 3 {
            return Err(FormatError {
                path: file_path.clone(),
                problem: format!("Unable to parse item line {}", line),
            });
        }
        let single_price = Receipt::parse_price(split[0]).ok_or(FormatError {
            path: file_path.clone(),
            problem: format!("Unable to parse item price {}", split[0]),
        })?;
        let consumer = String::from(*split.last().unwrap());
        if consumer.len() > 1 {
            return Err(FormatError {
                path: file_path.clone(),
                problem: format!("Unable to parse item consumer {}", consumer),
            });
        }
        let (name, count) = match split[1].strip_prefix("x") {
            Some(count_str) => {
                let count: u32 = count_str.parse::<u32>()
                    .map_err(|_| FormatError {
                        path: file_path.clone(),
                        problem: format!("Unable to parse item count / multiplier {}", split[1]),
                    })?;
                if split.len() < 4 {
                    return Err(FormatError {
                        path: file_path.clone(),
                        problem: format!("Unable to parse item line {}", line),
                    });
                }
                (split[2..(split.len() - 1)].join(" "), count)
            }
            None => (split[1..(split.len() - 1)].join(" "), 1),
        };
        Ok(Item { name, consumer, single_price, count })
    }
}

pub struct Receipt {
    pub file_path: PathBuf,
    pub purchaser: String,
    pub items: Vec<Item>,
}

impl Receipt {
    /// Attempts to parse a price written in dollars.cents format, and returns the total cents.
    /// # Examples
    /// ```
    /// use receipt_processor::receipt::Receipt;
    /// let result = Receipt::parse_price("1.50");
    /// assert_eq!(result, Some(150));
    ///
    /// let result = Receipt::parse_price(".69");
    /// assert_eq!(result, Some(69));
    ///
    /// let result = Receipt::parse_price("420");
    /// assert_eq!(result, Some(42000));
    ///
    /// let result = Receipt::parse_price("0");
    /// assert_eq!(result, Some(0));
    ///
    /// let result = Receipt::parse_price(".0");
    /// assert_eq!(result, Some(0));
    ///
    /// let result = Receipt::parse_price("0.00");
    /// assert_eq!(result, Some(0));
    ///
    /// let result = Receipt::parse_price(".3");
    /// assert_eq!(result, Some(30));
    ///
    /// let result = Receipt::parse_price(".");
    /// assert_eq!(result, None);
    ///
    /// let result = Receipt::parse_price("1,3");
    /// assert_eq!(result, None);
    ///
    /// let result = Receipt::parse_price("0.501");
    /// assert_eq!(result, None);
    ///
    /// let result = Receipt::parse_price("0.001");
    /// assert_eq!(result, None);
    ///
    /// let result = Receipt::parse_price("5.");
    /// assert_eq!(result, None);
    /// ```
    pub fn parse_price(str: &str) -> Option<u32> {
        let price_parts: Vec<&str> = str.split(".").collect();
        if price_parts.len() > 2 {
            return None;
        }
        let dollars = {
            if price_parts[0].is_empty() {
                Some(0)
            } else {
                price_parts[0].parse::<u32>().ok()
            }
        }?;
        let cents = {
            if price_parts.len() == 1 {
                // There was no `.` character
                if price_parts[0].is_empty() {
                    None
                } else {
                    Some(0)
                }
            } else {
                // `.` character is present, we require 1-2 digits after the dot
                if price_parts[1].is_empty() || price_parts[1].len() > 2 {
                    None
                } else {
                    let value = price_parts[1].parse::<u32>().ok();
                    if price_parts[1].len() == 1 {
                        // .3 = 30 cents
                        value.map(|c| c * 10)
                    } else {
                        value
                    }
                }
            }
        }?;

        Some(dollars * 100 + cents)
    }

    pub fn parse(file_path: PathBuf) -> Result<Receipt, ReceiptParseError> {
        let file = File::open(file_path.as_path())
            .map_err(|e| FileReadError { path: file_path.clone(), underlying_error: e.to_string() })?;
        let lines: Vec<String> = BufReader::new(&file).lines().collect::<io::Result<Vec<String>>>()
            .map_err(|e| FileReadError { path: file_path.clone(), underlying_error: e.to_string() })?;

        if lines.is_empty() {
            return Err(FileEmpty { path: file_path.clone() });
        }

        let purchase_line: Vec<&str> = lines[0].split(" ").collect();
        if purchase_line.len() != 2 || purchase_line[1] != "pirka" {
            return Err(FormatError {
                path: file_path.clone(),
                problem: String::from("The first line did not match the required \"<person> pirka\" format!"),
            });
        }
        let purchaser = String::from(purchase_line[0]);

        let mut items = Vec::new();
        for line in &lines[1..] {
            if line.starts_with("#") {
                continue; // Ignore comments
            }
            items.push(Item::parse(&file_path, line)?);
        }

        Ok(Receipt { file_path, purchaser, items })
    }

    pub fn total_spent(&self) -> u32 {
        let mut total = 0;
        for item in &self.items {
            total += item.total_price();
        }
        total
    }

    /// All people who received items from the purchase
    pub fn recipients(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        for item in &self.items {
            set.insert(item.consumer.clone());
        }
        set
    }
}


#[derive(Debug, Clone)]
pub enum ReceiptParseError {
    FileReadError { path: PathBuf, underlying_error: String },
    FileEmpty { path: PathBuf },
    FormatError { path: PathBuf, problem: String },
}

impl Display for ReceiptParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileReadError { path, underlying_error } =>
                write!(f, "Failed to read file {} - {}", path.to_str().unwrap(), underlying_error),
            FileEmpty { path } =>
                write!(f, "File {} was empty!", path.to_str().unwrap()),
            FormatError { path, problem } =>
                write!(f, "File {} was incorrectly formatted! {}", path.to_str().unwrap(), problem),
        }
    }
}

impl Error for ReceiptParseError {}