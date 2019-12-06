// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Defines the Row type which makes up lines in the grid

use std::ops::{Index, IndexMut};
use std::cmp::{min, max};
use std::slice;

use serde::{Deserialize, Serialize};

use crate::grid::GridCell;
use crate::index::Column;

/// A row in the grid
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Row<T> {
    inner: Vec<T>,
    columns: usize,
    template: T,
}

impl<T: PartialEq> PartialEq for Row<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Copy> Row<T> {
    /// Create a new row.
    #[inline]
    pub fn new(columns: Column, template: &T) -> Row<T>
    where
        T: GridCell,
    {
        Row { inner: Vec::with_capacity(columns.0), columns: columns.0, template: *template }
    }

    /// Create a new row from a vector of cells.
    #[inline]
    pub fn from_vec(vec: Vec<T>, template: &T, columns: Column) -> Row<T> {
        Row { inner: vec, columns: columns.0, template: *template }
    }

    /// Shrink the number of columns in this row.
    ///
    /// This will remove all cells which have been removed.
    pub fn shrink(&mut self, cols: Column) -> Option<Vec<T>>
    where
        T: GridCell,
    {
        self.columns = cols.0;

        if self.inner.len() <= cols.0 {
            return None;
        }

        // Split off cells for a new row
        let mut new_row = self.inner.split_off(cols.0);
        let index = new_row.iter().rposition(|c| !c.is_empty()).map(|i| i + 1).unwrap_or(0);
        new_row.truncate(index);

        if new_row.is_empty() {
            None
        } else {
            Some(new_row)
        }
    }

    /// Increase the number of columns in this row.
    #[inline]
    pub fn grow(&mut self, cols: Column) {
        self.columns = cols.0;
    }

    /// Clear the row and update the default cell.
    #[inline]
    pub fn reset(&mut self, template: &T)
    where
        T: GridCell,
    {
        self.template = *template;
        self.inner.clear();
    }

    /// Reset all cells after `at`.
    #[inline]
    pub fn reset_from(&mut self, at: usize, template: &T) {
        self.template = *template;
        self.inner.truncate(at);
    }

    /// Get a reference to the cell in the last column.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.columns == 0 {
            None
        } else {
            Some(self.inner.get(self.columns - 1).unwrap_or(&self.template))
        }
    }

    /// Get a mutable reference to the cell in the last column.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.fill(self.columns);
        self.inner.last_mut()
    }

    /// Returns an iterator that allows modifying each value.
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T>
    where
        T: Copy
    {
        self.fill(self.columns);
        self.inner.iter_mut()
    }

    /// Make sure the raw vector has at least `size` elements.
    #[inline]
    fn fill(&mut self, size: usize) {
        if self.inner.len() < size && size <= self.columns {
            for _ in self.inner.len()..size {
                self.inner.push(self.template);
            }
        }
    }

    /// Split off the front of the row.
    ///
    /// # Panics
    ///
    /// Panics if `at > self.len()`.
    #[inline]
    pub fn front_split_off(&mut self, at: usize) -> Vec<T> {
        // Assure at least `self.len()` can be split off without panic
        self.fill(min(self.columns, at));

        self.columns -= at;

        let mut split = self.inner.split_off(at);
        std::mem::swap(&mut split, &mut self.inner);
        split
    }
}

impl<T> Row<T> {
    /// Add new cells to the end of the row.
    #[inline]
    pub fn append(&mut self, vec: &mut Vec<T>) {
        self.inner.append(vec);
        self.columns = max(self.inner.len(), self.columns);
    }

    /// Add new cells to the start of the row.
    #[inline]
    pub fn append_front(&mut self, mut vec: Vec<T>) {
        vec.append(&mut self.inner);
        self.inner = vec;
        self.columns = max(self.inner.len(), self.columns);
    }

    /// Returns the number of cells in the row.
    #[inline]
    pub fn len(&self) -> usize {
        self.columns
    }

    /// Returns the number of non-empty cells in the row.
    #[inline]
    pub fn occupied(&self) -> Column
    where
        T: GridCell
    {
        Column(self.inner.iter().rposition(|cell| !cell.is_empty()).map(|o| o + 1).unwrap_or(0))
    }

    /// Returns `true` if all lines in the row are empty.
    #[inline]
    pub fn is_empty(&self) -> bool
    where
        T: GridCell,
    {
        self.inner.iter().all(GridCell::is_empty)
    }
}

impl<T> Index<Column> for Row<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Column) -> &T {
        self.inner.get(index.0).unwrap_or(&self.template)
    }
}

impl<T: Copy> IndexMut<Column> for Row<T> {
    #[inline]
    fn index_mut(&mut self, index: Column) -> &mut T {
        self.fill(index.0 + 1);
        &mut self.inner[index.0]
    }
}
