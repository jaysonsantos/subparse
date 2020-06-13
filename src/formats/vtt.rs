// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use self::errors::*;
use crate::{SubtitleEntry, SubtitleFileInterface};

use crate::errors::Result as SubtitleParserResult;

use failure::ResultExt;

use itertools::Itertools;

use crate::timetypes::{TimePoint, TimeSpan};

type Result<T> = std::result::Result<T, Error>;

/// Errors specific to `.srt`-parsing.
#[allow(missing_docs)]
pub mod errors {

    define_error!(Error, ErrorKind);

    #[derive(PartialEq, Debug, Fail)]
    pub enum ErrorKind {
        #[fail(display = "expected SubRip index line, found '{}'", line)]
        ExpectedIndexLine { line: String },

        #[fail(display = "expected SubRip timespan line, found '{}'", line)]
        ExpectedTimestampLine { line: String },

        #[fail(display = "parse error at line `{}`", line_num)]
        ErrorAtLine { line_num: usize },
    }
}

#[derive(Debug, Clone)]
/// Represents a `.vtt` file.
pub struct VttFile {
    v: Vec<VttLine>,
}

#[derive(Debug, Clone)]
/// A complete description of one `Vtt` subtitle line.
struct VttLine {
    /// start and end time of subtitle
    timespan: TimeSpan,

    /// index/number of line
    index: i64,

    /// the dialog/text lines of the `SrtLine`
    texts: Vec<String>,
}

impl VttFile {
    /// Parse a `.srt` subtitle string to `SrtFile`.
    pub fn parse(s: &str) -> SubtitleParserResult<VttFile> {
        Ok(Self::parse_file(s).with_context(|_| crate::ErrorKind::ParsingError)?)
    }
}

/// Implements parse functions.
impl VttFile {
    fn parse_file(i: &str) -> Result<VttFile> {
        unimplemented!();
    }
}

impl SubtitleFileInterface for VttFile {
    fn get_subtitle_entries(&self) -> SubtitleParserResult<Vec<SubtitleEntry>> {
        let timings = self
            .v
            .iter()
            .map(|line| SubtitleEntry::new(line.timespan, line.texts.iter().join("\n")))
            .collect();

        Ok(timings)
    }

    fn update_subtitle_entries(&mut self, new_subtitle_entries: &[SubtitleEntry]) -> SubtitleParserResult<()> {
        assert_eq!(self.v.len(), new_subtitle_entries.len()); // required by specification of this function

        for (line_ref, new_entry_ref) in self.v.iter_mut().zip(new_subtitle_entries) {
            line_ref.timespan = new_entry_ref.timespan;
            if let Some(ref text) = new_entry_ref.line {
                line_ref.texts = text.lines().map(str::to_string).collect();
            }
        }

        Ok(())
    }

    fn to_data(&self) -> SubtitleParserResult<Vec<u8>> {
        let timepoint_to_str =
            |t: TimePoint| -> String { format!("{:02}:{:02}:{:02}.{:03}", t.hours(), t.mins_comp(), t.secs_comp(), t.msecs_comp()) };
        let line_to_str = |line: &VttLine| -> String {
            format!(
                "{}\n{} --> {}\n{}\n\n",
                line.index,
                timepoint_to_str(line.timespan.start),
                timepoint_to_str(line.timespan.end),
                line.texts.join("\n")
            )
        };

        let mut output = vec![];
        output.extend_from_slice(b"WEBVTT\n\n");
        output.extend_from_slice(self.v.iter().map(line_to_str).collect::<String>().as_bytes());

        Ok(output)
    }
}

impl VttFile {
    /// Creates .vtt file from scratch.
    pub fn create(v: Vec<(TimeSpan, String)>) -> SubtitleParserResult<VttFile> {
        let file_parts = v
            .into_iter()
            .enumerate()
            .map(|(i, (ts, text))| VttLine {
                index: i as i64 + 1,
                timespan: ts,
                texts: text.lines().map(str::to_string).collect(),
            })
            .collect();

        Ok(VttFile { v: file_parts })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_vtt_test() {
        use crate::timetypes::{TimePoint, TimeSpan};
        use crate::SubtitleFileInterface;

        let lines = vec![
            (
                TimeSpan::new(TimePoint::from_msecs(1500), TimePoint::from_msecs(3700)),
                "line1".to_string(),
            ),
            (
                TimeSpan::new(TimePoint::from_msecs(4500), TimePoint::from_msecs(8700)),
                "line2".to_string(),
            ),
        ];
        let file = super::VttFile::create(lines).unwrap();

        // generate file
        let data_string = String::from_utf8(file.to_data().unwrap()).unwrap();
        let expected = "WEBVTT\n\n1\n00:00:01.500 --> 00:00:03.700\nline1\n\n2\n00:00:04.500 --> 00:00:08.700\nline2\n\n".to_string();
        assert_eq!(data_string, expected);
    }
}
// TODO: parser tests
