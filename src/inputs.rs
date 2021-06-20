use chrono::Datelike;
use resolution::DateResolution as DateResolutionTrait;
use std::{
    cmp, collections,
    convert::{self, TryFrom},
    error, fmt, marker, num,
};

#[derive(Debug)]
pub struct SelectError {
    selected: String,
}

impl SelectError {
    pub fn new(selected: String) -> SelectError {
        SelectError { selected }
    }
}

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value {} is not in the list of allowed options",
            self.selected
        )
    }
}

impl error::Error for SelectError {}

pub type BasicSelect = Select<convert::Infallible, String>;

pub struct Select<
    E: error::Error + Sync + Send + 'static,
    O: fmt::Display + Ord + std::str::FromStr<Err = E>,
> {
    input: String,
    options: collections::BTreeSet<O>,
    o: marker::PhantomData<O>,
}

impl<
        E: error::Error + Sync + Send + 'static,
        O: fmt::Display + Ord + std::str::FromStr<Err = E>,
    > Select<E, O>
{
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(data: O, options: collections::BTreeSet<O>) -> Select<E, O> {
        Select {
            input: data.to_string(),
            options,
            o: marker::PhantomData,
        }
    }
    pub fn iter_options(&self) -> impl Iterator<Item = &O> {
        self.options.iter()
    }
}

impl<
        E: error::Error + Sync + Send + 'static,
        O: fmt::Display + Ord + std::str::FromStr<Err = E>,
    > crate::UserInput for Select<E, O>
{
    type Output = O;
    type Input = String;

    fn set_value(&mut self, data: O) {
        self.input = data.to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = self.input.parse()?;
        if self.options.contains(&parsed) {
            Ok(parsed)
        } else {
            Err(SelectError {
                selected: self.input.to_string(),
            }
            .into())
        }
    }
}

pub struct RelationalSelect<
    E: error::Error + Sync + Send + 'static,
    K: fmt::Display + std::str::FromStr<Err = E> + cmp::Ord,
    V: fmt::Display,
> {
    input: String,
    options: collections::BTreeMap<K, V>,
    k: marker::PhantomData<K>,
}

impl<
        E: error::Error + Sync + Send + 'static,
        K: fmt::Display + std::str::FromStr<Err = E> + cmp::Ord,
        V: fmt::Display,
    > RelationalSelect<E, K, V>
{
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(input: String, options: collections::BTreeMap<K, V>) -> RelationalSelect<E, K, V> {
        RelationalSelect {
            input,
            options,
            k: marker::PhantomData,
        }
    }
    pub fn iter_options(&self) -> impl Iterator<Item = (&K, &V)> {
        self.options.iter()
    }
}

impl<
        E: error::Error + Sync + Send + 'static,
        K: fmt::Display + std::str::FromStr<Err = E> + cmp::Ord,
        V: fmt::Display,
    > crate::UserInput for RelationalSelect<E, K, V>
{
    type Output = K;
    type Input = String;

    fn set_value(&mut self, data: K) {
        self.input = data.to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let key = self.input.parse()?;

        if self.options.contains_key(&key) {
            Ok(key)
        } else {
            Err(SelectError {
                selected: self.input.to_string(),
            }
            .into())
        }
    }
}

pub struct Scalar<O, E>
where
    O: std::str::FromStr<Err = E> + fmt::Display,
    E: error::Error + Sync + Send + 'static,
{
    input: String,
    validations: crate::Validations<O>,
    o: marker::PhantomData<O>,
}

impl<O, E> Scalar<O, E>
where
    O: std::str::FromStr<Err = E> + fmt::Display,
    E: error::Error + Sync + Send + 'static,
{
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(data: &O, validations: crate::Validations<O>) -> Scalar<O, E> {
        Scalar {
            input: data.to_string(),
            validations,
            o: marker::PhantomData,
        }
    }
}

impl<O, E> Default for Scalar<O, E>
where
    O: std::str::FromStr<Err = E> + Default + fmt::Display,
    E: error::Error + Sync + Send + 'static,
{
    fn default() -> Scalar<O, E> {
        Scalar::new(&O::default(), crate::Validations::new())
    }
}

impl<O, E> crate::UserInput for Scalar<O, E>
where
    O: std::str::FromStr<Err = E> + fmt::Display,
    E: error::Error + Sync + Send + 'static,
{
    type Output = O;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = self.input.parse()?;
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub type Integer<I> = Scalar<I, num::ParseIntError>;
pub type Decimal = Scalar<rust_decimal::Decimal, rust_decimal::Error>;
pub type Text = Scalar<String, convert::Infallible>;

pub struct NaiveDate {
    input: String,
    format: &'static str,
    validations: crate::Validations<chrono::NaiveDate>,
}

impl NaiveDate {
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(
        data: chrono::NaiveDate,
        format: &'static str,
        validations: crate::Validations<chrono::NaiveDate>,
    ) -> NaiveDate {
        NaiveDate {
            input: data.format(format).to_string(),
            format,
            validations,
        }
    }
}

impl Default for NaiveDate {
    fn default() -> NaiveDate {
        NaiveDate::new(
            chrono::Utc::now().date().naive_utc(),
            "%Y-%m-%d",
            crate::Validations::new(),
        )
    }
}

impl crate::UserInput for NaiveDate {
    type Output = chrono::NaiveDate;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.format(self.format).to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = chrono::NaiveDate::parse_from_str(&self.input, self.format)?;
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct Date {
    input: String,
    format: &'static str,
    validations: crate::Validations<resolution::Date>,
}

impl Default for Date {
    fn default() -> Date {
        Date::new(
            chrono::Utc::now().date().naive_utc().into(),
            "%Y-%m-%d",
            crate::Validations::new(),
        )
    }
}

impl Date {
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(
        data: resolution::Date,
        format: &'static str,
        validations: crate::Validations<resolution::Date>,
    ) -> Date {
        Date {
            input: data.start().format(format).to_string(),
            format,
            validations,
        }
    }
}

impl crate::UserInput for Date {
    type Output = resolution::Date;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.start().format(self.format).to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = chrono::NaiveDate::parse_from_str(&self.input, self.format)?.into();
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct Year {
    input: String,
    validations: crate::Validations<resolution::Year>,
}

impl Default for Year {
    fn default() -> Year {
        Year::new(
            resolution::Year::new(chrono::Utc::now().date().year()),
            crate::Validations::new(),
        )
    }
}

impl Year {
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(data: resolution::Year, validations: crate::Validations<resolution::Year>) -> Year {
        Year {
            input: data.to_string(),
            validations,
        }
    }
}

impl crate::UserInput for Year {
    type Output = resolution::Year;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.year_num().to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = resolution::Year::new(self.input.parse()?);
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct RelativeMonth {
    input: String,
    validations: crate::Validations<u32>,
}

impl Default for RelativeMonth {
    fn default() -> RelativeMonth {
        RelativeMonth::new(chrono::Utc::now().month(), crate::Validations::new())
    }
}

impl RelativeMonth {
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(data: u32, validations: crate::Validations<u32>) -> RelativeMonth {
        RelativeMonth {
            input: data.to_string(),
            validations,
        }
    }
}

impl crate::UserInput for RelativeMonth {
    type Output = u32;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = self.input.parse()?;
        if parsed < 1 || parsed > 12 {
            return Err(crate::Error::Validation(
                vec![format!(
                    "Month number should be between 1 and 12 but was {}",
                    parsed
                )]
                .into(),
            ));
        };
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct Month {
    year: Year,
    month: RelativeMonth,
    validations: crate::Validations<resolution::Month>,
}

impl Default for Month {
    fn default() -> Month {
        Month {
            validations: crate::Validations::new(),
            year: Year::default(),
            month: RelativeMonth::default(),
        }
    }
}

pub enum MonthMsg {
    Year(String),
    Month(String),
}

impl Month {
    pub fn get_year(&self) -> &Year {
        &self.year
    }
    pub fn get_month(&self) -> &RelativeMonth {
        &self.month
    }
    pub fn new(
        input: resolution::Month,
        year_validations: crate::Validations<resolution::Year>,
        month_validations: crate::Validations<u32>,
        validations: crate::Validations<resolution::Month>,
    ) -> Month {
        Month {
            year: Year::new(input.year(), year_validations),
            month: RelativeMonth::new(input.month_num(), month_validations),
            validations,
        }
    }
}

impl crate::UserInput for Month {
    type Output = resolution::Month;
    type Input = MonthMsg;
    fn set_value(&mut self, data: Self::Output) {
        self.year.set_value(data.year());
        self.month.set_value(data.month_num());
    }
    fn update(&mut self, input: Self::Input) {
        match input {
            MonthMsg::Year(y) => self.year.input = y,
            MonthMsg::Month(m) => self.month.input = m,
        }
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let year = self.year.parse()?;
        let month = self.month.parse()?;
        let parsed =
            resolution::Month::from_date(chrono::NaiveDate::from_ymd(year.year_num(), month, 1));
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct RelativeQuarter {
    input: String,
    validations: crate::Validations<u32>,
}

impl RelativeQuarter {
    pub fn get_input(&self) -> &str {
        &self.input
    }
    pub fn new(data: u32, validations: crate::Validations<u32>) -> RelativeQuarter {
        // we can accept data other than 1..4 as it will just result in a failed .parse()
        RelativeQuarter {
            input: data.to_string(),
            validations,
        }
    }
}

impl crate::UserInput for RelativeQuarter {
    type Output = u32;
    type Input = String;
    fn set_value(&mut self, data: Self::Output) {
        self.input = data.to_string();
    }
    fn update(&mut self, input: Self::Input) {
        self.input = input;
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let parsed = self.input.parse()?;
        if parsed < 1 || parsed > 4 {
            return Err(crate::Error::Validation(
                vec![format!(
                    "Quarter number should be between 1 and 4 but was {}",
                    parsed
                )]
                .into(),
            ));
        };
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

impl Default for RelativeQuarter {
    fn default() -> RelativeQuarter {
        RelativeQuarter::new(
            (chrono::Utc::now().month() + 2) / 3,
            crate::Validations::new(),
        )
    }
}

pub struct Quarter {
    year: Year,
    quarter: RelativeQuarter,
    validations: crate::Validations<resolution::Quarter>,
}

impl Default for Quarter {
    fn default() -> Quarter {
        Quarter {
            validations: crate::Validations::new(),
            year: Year::default(),
            quarter: RelativeQuarter::default(),
        }
    }
}

pub enum QuarterMsg {
    Year(String),
    Quarter(String),
}

impl Quarter {
    pub fn get_year(&self) -> &Year {
        &self.year
    }
    pub fn get_quarter(&self) -> &RelativeQuarter {
        &self.quarter
    }
    pub fn new(
        input: resolution::Quarter,
        year_validations: crate::Validations<resolution::Year>,
        quarter_validations: crate::Validations<u32>,
        validations: crate::Validations<resolution::Quarter>,
    ) -> Quarter {
        Quarter {
            year: Year::new(input.year(), year_validations),
            quarter: RelativeQuarter::new(input.quarter_num(), quarter_validations),
            validations,
        }
    }
}

impl crate::UserInput for Quarter {
    type Output = resolution::Quarter;
    type Input = QuarterMsg;
    fn set_value(&mut self, data: Self::Output) {
        self.year.set_value(data.year());
        self.quarter.set_value(data.quarter_num());
    }
    fn update(&mut self, input: Self::Input) {
        match input {
            QuarterMsg::Year(y) => self.year.input = y,
            QuarterMsg::Quarter(m) => self.quarter.input = m,
        }
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let year = self.year.parse()?;
        let quarter = self.quarter.parse()?;
        let parsed = resolution::Quarter::from_date(chrono::NaiveDate::from_ymd(
            year.year_num(),
            quarter * 3 - 2,
            1,
        ));
        self.validations.validate(&parsed)?;
        Ok(parsed)
    }
}

pub struct DateResolution<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    input: I,
    _r: marker::PhantomData<R>,
}

impl<I, R> DateResolution<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    pub fn new(input: I) -> DateResolution<I, R> {
        DateResolution {
            _r: marker::PhantomData,
            input,
        }
    }
    pub fn from_data(data: R) -> DateResolution<I, R> {
        let mut input = I::default();
        input.set_value(data);
        DateResolution::new(input)
    }
    pub fn get_input(&self) -> &I {
        &self.input
    }
}

impl<I, R> From<I> for DateResolution<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    fn from(input: I) -> DateResolution<I, R> {
        DateResolution::new(input)
    }
}

impl<I, R> Default for DateResolution<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    fn default() -> DateResolution<I, R> {
        DateResolution::new(I::default())
    }
}

impl<I, R> crate::UserInput for DateResolution<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    type Output = I::Output;
    type Input = I::Input;
    fn set_value(&mut self, data: Self::Output) {
        self.input.set_value(data);
    }
    fn update(&mut self, input: Self::Input) {
        self.input.update(input);
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        Ok(self.input.parse()?)
    }
}

pub struct TimeRange<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    date_resolution: DateResolution<I, R>,
    length: Integer<u32>,
    length_validations: crate::Validations<u32>,
    range_validations: crate::Validations<resolution::TimeRange<R>>,
    _r: marker::PhantomData<R>,
}

fn greater_than_zero(input: &u32) -> crate::ValidationResult {
    if input > &0 {
        Ok(())
    } else {
        Err("Input must be greater than zero".to_string())
    }
}

impl<I, R> TimeRange<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    pub fn new(
        data: resolution::TimeRange<R>,
        mut dr_input: I,
        length_validations: crate::Validations<u32>,
        range_validations: crate::Validations<resolution::TimeRange<R>>,
    ) -> TimeRange<I, R> {
        dr_input.set_value(data.start());
        let date_resolution = DateResolution::new(dr_input);
        TimeRange {
            _r: marker::PhantomData,
            date_resolution,
            length: Integer::new(
                &u32::try_from(data.len()).unwrap(),
                crate::Validations::from_vec(vec![greater_than_zero]),
            ),
            length_validations,
            range_validations,
        }
    }
    pub fn get_length(&self) -> &Integer<u32> {
        &self.length
    }
    pub fn get_date_resolution(&self) -> &DateResolution<I, R> {
        &self.date_resolution
    }
}
impl<I, R> Default for TimeRange<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    fn default() -> TimeRange<I, R> {
        TimeRange {
            _r: marker::PhantomData,
            date_resolution: DateResolution::default(),
            length: Integer::new(&1, crate::Validations::from_vec(vec![greater_than_zero])),
            length_validations: crate::Validations::new(),
            range_validations: crate::Validations::new(),
        }
    }
}

pub enum TimeRangeMsg<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    DateResolution {
        input: I::Input,
        _r: marker::PhantomData<R>,
    },
    Length(String),
}

impl<I, R> crate::UserInput for TimeRange<I, R>
where
    R: resolution::DateResolution,
    I: crate::UserInput<Output = R> + Default,
{
    type Output = resolution::TimeRange<R>;
    type Input = TimeRangeMsg<I, R>;
    fn set_value(&mut self, data: Self::Output) {
        self.length.set_value(u32::try_from(data.len()).unwrap());
        self.date_resolution.set_value(data.start());
    }
    fn update(&mut self, input: Self::Input) {
        match input {
            TimeRangeMsg::DateResolution { input, .. } => self.date_resolution.update(input),
            TimeRangeMsg::Length(input) => self.length.update(input),
        }
    }
    fn parse(&self) -> crate::Result<Self::Output> {
        let start = self.date_resolution.parse()?;
        let len = self.length.parse()?;
        self.length_validations.validate(&len)?;
        let range = resolution::TimeRange::new(start, len);
        self.range_validations.validate(&range)?;
        Ok(range)
    }
}
