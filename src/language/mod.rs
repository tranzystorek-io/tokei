pub mod language_type;
pub mod languages;
mod syntax;

use std::{collections::BTreeMap, mem, ops::AddAssign};

pub use self::{language_type::*, languages::Languages};

use crate::{
    sort::Sort::{self, *},
    stats::Report,
};

/// A struct representing statistics about a single Language.
#[derive(Clone, Debug, Deserialize, Default, PartialEq, Serialize)]
pub struct Language {
    /// The total number of blank lines.
    pub blanks: usize,
    /// The total number of lines of code.
    pub code: usize,
    /// The total number of comments(both single, and multi-line)
    pub comments: usize,
    /// A collection of statistics of individual files.
    pub reports: Vec<Report>,
    /// A map of any languages found in the reports.
    pub children: BTreeMap<LanguageType, Vec<Report>>,
    /// Whether this language had problems with file parsing
    pub inaccurate: bool,
}

impl Language {
    /// Constructs a new empty Language with the comments provided.
    ///
    /// ```
    /// # use tokei::*;
    /// let mut rust = Language::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of lines.
    #[inline]
    pub fn lines(&self) -> usize {
        self.blanks + self.code + self.comments
    }

    /// Add a `Report` to the Language. This will not update the totals in the
    /// Language struct.
    pub fn add_report(&mut self, report: Report) {
        for (lang, stats) in &report.stats.contexts {
            let mut new_report = Report::new(report.name.clone());
            new_report.stats = stats.clone();

            self.children.entry(*lang).or_default().push(new_report);
        }

        self.reports.push(report);
    }

    /// Marks this language as possibly not reflecting correct stats.
    #[inline]
    pub fn mark_inaccurate(&mut self) {
        self.inaccurate = true;
    }

    /// Creates a new `Language` from `self`, which is a summarised version
    /// of the language that doesn't contain any children. It will count
    /// non-blank lines in child languages as code unless the child language is
    /// considered "literate" then it will be counted as comments.
    pub fn summarise(&self) -> Language {
        let mut summary = self.clone();

        for reports in self.children.values() {
            for stats in reports.iter().map(|r| r.stats.summarise()) {
                summary.comments += stats.comments;
                summary.code += stats.code;
                summary.blanks += stats.blanks;
            }
        }

        summary
    }

    /// Totals up the statistics of the `Stat` structs currently contained in
    /// the language.
    ///
    /// ```no_run
    /// use std::{collections::BTreeMap, path::PathBuf};
    /// use tokei::Language;
    ///
    /// let mut language = Language::new();
    ///
    /// // Add stats...
    ///
    /// assert_eq!(0, language.lines());
    ///
    /// language.total();
    ///
    /// assert_eq!(10, language.lines());
    /// ```
    pub fn total(&mut self) {
        let mut blanks = 0;
        let mut code = 0;
        let mut comments = 0;

        for report in &self.reports {
            blanks += report.stats.blanks;
            code += report.stats.code;
            comments += report.stats.comments;
        }

        self.blanks = blanks;
        self.code = code;
        self.comments = comments;
    }

    /// Checks if the language is empty. Empty meaning it doesn't have any
    /// statistics.
    ///
    /// ```
    /// # use tokei::*;
    /// let rust = Language::new();
    ///
    /// assert!(rust.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.code == 0 && self.comments == 0 && self.blanks == 0 && self.children.is_empty()
    }

    /// Sorts each of the `Report`s contained in the language based
    /// on what category is provided.
    ///
    /// ```no_run
    /// use std::{collections::BTreeMap, path::PathBuf};
    /// use tokei::{Language, Sort};
    ///
    /// let mut language = Language::new();
    ///
    /// // Add stats...
    ///
    /// language.sort_by(Sort::Lines);
    /// assert_eq!(20, language.reports[0].stats.lines());
    ///
    /// language.sort_by(Sort::Code);
    /// assert_eq!(8, language.reports[0].stats.code);
    /// ```
    pub fn sort_by(&mut self, category: Sort) {
        match category {
            Blanks => self
                .reports
                .sort_by(|a, b| b.stats.blanks.cmp(&a.stats.blanks)),
            Comments => self
                .reports
                .sort_by(|a, b| b.stats.comments.cmp(&a.stats.comments)),
            Code => self.reports.sort_by(|a, b| b.stats.code.cmp(&a.stats.code)),
            Files => self.reports.sort_by(|a, b| a.name.cmp(&b.name)),
            Lines => self
                .reports
                .sort_by(|a, b| b.stats.lines().cmp(&a.stats.lines())),
        }
    }
}

impl AddAssign for Language {
    fn add_assign(&mut self, mut rhs: Self) {
        self.comments += rhs.comments;
        self.blanks += rhs.blanks;
        self.code += rhs.code;
        self.reports
            .extend(mem::replace(&mut rhs.reports, Vec::new()));
        self.children
            .extend(mem::replace(&mut rhs.children, BTreeMap::new()));
        self.inaccurate |= rhs.inaccurate
    }
}
