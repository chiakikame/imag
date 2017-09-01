//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

/// A Habit is a "template" of a habit. A user may define a habit "Eat vegetable".
/// If the user ate a vegetable, she should create a HabitInstance from the Habit with the
/// appropriate date (and optionally a comment) set.
pub trait Habit {

    /// Check whether the instance is a habit by checking its headers for the habit data
    fn is_habit(&self) -> Result<bool>;

    fn create_instance(&self) -> HabitCreator;

    fn instances(&self, &Store) -> HabitInstanceIterator<'a>;

}

pub struct HabitCreator {
    name: Option<String>,
    comment: Option<String>,
    date: Option<NaiveDate>,
}

impl HabitBuilder {

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn with_comment(&mut self, comment: String) -> &mut Self {
        self.comment = Some(comment);
        self
    }

    pub fn with_date(&mut self, date: NaiveDate) -> &mut Self {
        self.date = Some(date);
        self
    }

    pub fn build<'a>(self, store: &'a Store) -> Result<FileLockEntry<'a>> {
    }

}

