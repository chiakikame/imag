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

/// Extension trait for libimagstore::store::Store which is basically our Habit-Store
pub trait HabitStore<'a> {

    /// Create a new habit
    fn create_habit(&self) -> Result<HabitBuilder<'a>>;

    /// Get an iterator over all habits
    fn all_habits(&self) -> HabitStoreIdIterator<'a>;

}


/// A specification of a habit, can be turned into a Habit via the store::HabitStore trait
pub struct HabitBuilder<'a> {
    store: &'a Store,
    name: Option<String>,
    comment: Option<String>,
    question: Option<String>,
    recurrency: Option<Recurrence>,
}

impl<'a> HabitBuilder<'a> {
    fn new(&'a Store) -> HabitBuilder<'a> {
        HabitBuilder {
            store      : store,
            name       : None,
            comment    : None,
            recurrency : None,
        }
    }

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn with_comment(&mut self, comment: String) -> &mut Self {
        self.comment = Some(comment);
        self
    }

    pub fn with_question(&mut self, question: String) -> &mut Self {
        self.question = Some(question);
        self
    }

    pub fn with_recurrence(&mut self, recurrence: Recurrence) -> &mut Self {
        self.recurrence = Some(recurrence);
        self
    }

    pub fn build<'a>(self, store: &'a Store) -> Result<FileLockEntry<'a>> {
    }

}

