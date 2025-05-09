mod days_of_month;
mod days_of_week;
mod hours;
mod minutes;
mod months;
mod seconds;
mod years;

use std::{borrow::Cow, collections::btree_set, iter, ops::RangeBounds};

pub use self::{
    days_of_month::DaysOfMonth, days_of_week::DaysOfWeek, hours::Hours, minutes::Minutes,
    months::Months, seconds::Seconds, years::Years,
};
use crate::{
    error::*,
    ordinal::{Ordinal, OrdinalSet},
    specifier::{RootSpecifier, Specifier},
};

pub struct OrdinalIter<'a> {
    set_iter: btree_set::Iter<'a, Ordinal>,
}

impl Iterator for OrdinalIter<'_> {
    type Item = Ordinal;
    fn next(&mut self) -> Option<Ordinal> {
        self.set_iter.next().copied()
    }
}

impl DoubleEndedIterator for OrdinalIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.set_iter.next_back().copied()
    }
}

pub struct OrdinalRangeIter<'a> {
    range_iter: btree_set::Range<'a, Ordinal>,
}

impl Iterator for OrdinalRangeIter<'_> {
    type Item = Ordinal;
    fn next(&mut self) -> Option<Ordinal> {
        self.range_iter.next().copied()
    }
}

impl DoubleEndedIterator for OrdinalRangeIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range_iter.next_back().copied()
    }
}

/// Methods exposing a schedule's configured ordinals for each individual unit
/// of time.
///
/// # Example
///
/// ```rust
/// use std::{
///     ops::Bound::{Excluded, Included},
///     str::FromStr,
/// };
///
/// use jiff_cron::{Schedule, TimeUnitSpec};
///
/// let expression = "* * * * * * 2015-2044";
/// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
///
/// // Membership
/// assert_eq!(true, schedule.years().includes(2031));
/// assert_eq!(false, schedule.years().includes(1969));
///
/// // Number of years specified
/// assert_eq!(30, schedule.years().count());
///
/// // Iterator
/// let mut years_iter = schedule.years().iter();
/// assert_eq!(Some(2015), years_iter.next());
/// assert_eq!(Some(2016), years_iter.next());
/// // ...
///
/// // Range Iterator
/// let mut five_year_plan = schedule.years().range((Included(2017), Excluded(2017 + 5)));
/// assert_eq!(Some(2017), five_year_plan.next());
/// assert_eq!(Some(2018), five_year_plan.next());
/// assert_eq!(Some(2019), five_year_plan.next());
/// assert_eq!(Some(2020), five_year_plan.next());
/// assert_eq!(Some(2021), five_year_plan.next());
/// assert_eq!(None, five_year_plan.next());
/// ```
pub trait TimeUnitSpec {
    /// Returns true if the provided ordinal was included in the schedule spec
    /// for the unit of time being described.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use jiff_cron::{Schedule, TimeUnitSpec};
    ///
    /// let expression = "* * * * * * 2015-2044";
    /// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
    ///
    /// // Membership
    /// assert_eq!(true, schedule.years().includes(2031));
    /// assert_eq!(false, schedule.years().includes(2004));
    /// ```
    fn includes(&self, ordinal: Ordinal) -> bool;

    /// Provides an iterator which will return each included ordinal for this
    /// schedule in order from lowest to highest.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use jiff_cron::{Schedule, TimeUnitSpec};
    ///
    /// let expression = "* * * * 5-8 * *";
    /// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
    ///
    /// // Iterator
    /// let mut summer = schedule.months().iter();
    /// assert_eq!(Some(5), summer.next());
    /// assert_eq!(Some(6), summer.next());
    /// assert_eq!(Some(7), summer.next());
    /// assert_eq!(Some(8), summer.next());
    /// assert_eq!(None, summer.next());
    /// ```
    fn iter(&self) -> OrdinalIter<'_>;

    /// Provides an iterator which will return each included ordinal within the
    /// specified range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::{
    ///     ops::Bound::{Excluded, Included},
    ///     str::FromStr,
    /// };
    ///
    /// use jiff_cron::{Schedule, TimeUnitSpec};
    ///
    /// let expression = "* * * 1,15 * * *";
    /// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
    ///
    /// // Range Iterator
    /// let mut mid_month_paydays = schedule.days_of_month().range((Included(10), Included(20)));
    /// assert_eq!(Some(15), mid_month_paydays.next());
    /// assert_eq!(None, mid_month_paydays.next());
    /// ```
    fn range<R>(&self, range: R) -> OrdinalRangeIter<'_>
    where
        R: RangeBounds<Ordinal>;

    /// Returns the number of ordinals included in the associated schedule
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use jiff_cron::{Schedule, TimeUnitSpec};
    ///
    /// let expression = "* * * 1,15 * * *";
    /// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
    ///
    /// assert_eq!(2, schedule.days_of_month().count());
    /// ```
    fn count(&self) -> u32;

    /// Checks if this TimeUnitSpec is defined as all possibilities (thus
    /// created with a '*', '?' or in the case of weekdays '1-7')
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use jiff_cron::{Schedule, TimeUnitSpec};
    ///
    /// let expression = "* * * 1,15 * * *";
    /// let schedule = Schedule::from_str(expression).expect("Failed to parse expression.");
    ///
    /// assert_eq!(false, schedule.days_of_month().is_all());
    /// assert_eq!(true, schedule.months().is_all());
    /// ```
    fn is_all(&self) -> bool;
}

impl<T> TimeUnitSpec for T
where
    T: TimeUnitField,
{
    fn includes(&self, ordinal: Ordinal) -> bool {
        self.ordinals().contains(&ordinal)
    }
    fn iter(&self) -> OrdinalIter<'_> {
        OrdinalIter {
            set_iter: TimeUnitField::ordinals(self).iter(),
        }
    }
    fn range<R>(&'_ self, range: R) -> OrdinalRangeIter<'_>
    where
        R: RangeBounds<Ordinal>,
    {
        OrdinalRangeIter {
            range_iter: TimeUnitField::ordinals(self).range(range),
        }
    }
    fn count(&self) -> u32 {
        self.ordinals().len() as u32
    }

    fn is_all(&self) -> bool {
        let max_supported_ordinals = Self::inclusive_max() - Self::inclusive_min() + 1;
        self.ordinals().len() == max_supported_ordinals as usize
    }
}

pub trait TimeUnitField
where
    Self: Sized,
{
    fn from_optional_ordinal_set(ordinal_set: Option<OrdinalSet>) -> Self;
    fn name() -> Cow<'static, str>;
    fn inclusive_min() -> Ordinal;
    fn inclusive_max() -> Ordinal;
    fn ordinals(&self) -> &OrdinalSet;

    fn from_ordinal(ordinal: Ordinal) -> Self {
        Self::from_ordinal_set(iter::once(ordinal).collect())
    }

    fn supported_ordinals() -> OrdinalSet {
        (Self::inclusive_min()..Self::inclusive_max() + 1).collect()
    }

    fn all() -> Self {
        Self::from_optional_ordinal_set(None)
    }

    fn from_ordinal_set(ordinal_set: OrdinalSet) -> Self {
        Self::from_optional_ordinal_set(Some(ordinal_set))
    }

    fn ordinal_from_name(name: &str) -> Result<Ordinal, Error> {
        Err(ErrorKind::Expression(format!(
            "The '{}' field does not support using names. '{}' specified.",
            Self::name(),
            name
        ))
        .into())
    }
    fn validate_ordinal(ordinal: Ordinal) -> Result<Ordinal, Error> {
        //println!("validate_ordinal for {} => {}", Self::name(), ordinal);
        match ordinal {
            i if i < Self::inclusive_min() => Err(ErrorKind::Expression(format!(
                "{} must be greater than or equal to {}. ('{}' specified.)",
                Self::name(),
                Self::inclusive_min(),
                i
            ))
            .into()),
            i if i > Self::inclusive_max() => Err(ErrorKind::Expression(format!(
                "{} must be less than {}. ('{}' specified.)",
                Self::name(),
                Self::inclusive_max(),
                i
            ))
            .into()),
            i => Ok(i),
        }
    }

    fn ordinals_from_specifier(specifier: &Specifier) -> Result<OrdinalSet, Error> {
        use self::Specifier::*;
        //println!("ordinals_from_specifier for {} => {:?}", Self::name(), specifier);
        match *specifier {
            All => Ok(Self::supported_ordinals()),
            Point(ordinal) => Ok(([ordinal]).iter().cloned().collect()),
            Range(start, end) => {
                match (Self::validate_ordinal(start), Self::validate_ordinal(end)) {
                    (Ok(start), Ok(end)) if start <= end => Ok((start..end + 1).collect()),
                    _ => Err(ErrorKind::Expression(format!(
                        "Invalid range for {}: {}-{}",
                        Self::name(),
                        start,
                        end
                    ))
                    .into()),
                }
            }
            NamedRange(ref start_name, ref end_name) => {
                let start = Self::ordinal_from_name(start_name)?;
                let end = Self::ordinal_from_name(end_name)?;
                match (Self::validate_ordinal(start), Self::validate_ordinal(end)) {
                    (Ok(start), Ok(end)) if start <= end => Ok((start..end + 1).collect()),
                    _ => Err(ErrorKind::Expression(format!(
                        "Invalid named range for {}: {}-{}",
                        Self::name(),
                        start_name,
                        end_name
                    ))
                    .into()),
                }
            }
        }
    }

    fn ordinals_from_root_specifier(root_specifier: &RootSpecifier) -> Result<OrdinalSet, Error> {
        let ordinals = match root_specifier {
            RootSpecifier::Specifier(specifier) => Self::ordinals_from_specifier(specifier)?,
            RootSpecifier::Period(_, 0) => Err(ErrorKind::Expression(
                "range step cannot be zero".to_string(),
            ))?,
            RootSpecifier::Period(start, step) => {
                if *step < 1 || *step > Self::inclusive_max() {
                    return Err(ErrorKind::Expression(format!(
                        "{} must be between 1 and {}. ('{}' specified.)",
                        Self::name(),
                        Self::inclusive_max(),
                        step,
                    ))
                    .into());
                }

                let base_set = match start {
                    // A point prior to a period implies a range whose start is the specified
                    // point and terminating inclusively with the inclusive max
                    Specifier::Point(start) => {
                        let start = Self::validate_ordinal(*start)?;
                        (start..=Self::inclusive_max()).collect()
                    }
                    specifier => Self::ordinals_from_specifier(specifier)?,
                };
                base_set.into_iter().step_by(*step as usize).collect()
            }
            RootSpecifier::NamedPoint(ref name) => ([Self::ordinal_from_name(name)?])
                .iter()
                .cloned()
                .collect::<OrdinalSet>(),
        };
        Ok(ordinals)
    }
}
