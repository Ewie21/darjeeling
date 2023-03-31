use std::fmt::Debug;

use serde::Serialize;

use crate::types::Types;

#[derive(Debug, Serialize)]
pub struct DataFrame<'a> {
    frame: Vec<Vec<Point<'a>>>,
    row_labels: Vec<&'a str>,
    col_labels: Vec<&'a str>
}

#[derive(Debug, Clone, Serialize)]
pub struct Point<'a> {
    row: usize,
    col: usize,
    val: Types<'a>
}

impl<'a> DataFrame<'a> {

    pub fn new(frame: Vec<Vec<Types<'a>>>, row_labels: Vec<&'a str>, col_labels: Vec<&'a str>) -> DataFrame<'a> {

        let mut proper_frame: Vec<Vec<Point<'a>>> = vec![];
        // For every column
        for i in 0..frame.len() {

            // proper_frame.push(Point::point_vector(i, frame[i].clone()));
            proper_frame.push(vec![]);
            // For every row in that column
            for j in 0..frame[i].len() {
                proper_frame[i].push(Point::new( j, i, frame[i][j]));
            }
        }

        DataFrame { frame: proper_frame, row_labels, col_labels }
    }

    pub fn value_at_index(&self, row: usize, col    : usize) -> &Types {

        &&self.frame[col][row].val
    }

    pub fn index_at_labels(&self, row: &str, col: &str) -> Result<(usize, usize), &str>{

        let row_index = match DataFrame::<'a>::label_index(row, &self.row_labels) {

            Ok(i) => i,
            Err(error) => return Err(error)
        };
        let col_index = match DataFrame::<'a>::label_index(col, &self.col_labels) {

            Ok(i) => i,
            Err(error) => return Err(error)

        };

        Ok((row_index, col_index))
    }

    pub fn value_at_labels(&self, row: &str, col: &str) ->  Result<&Types, &str> {

        let row_index: usize = match DataFrame::<'a>::label_index(row, &self.row_labels) {

            Ok(i) => i,
            Err(error) => return Err(error)
        };
        let col_index: usize = match DataFrame::<'a>::label_index(col, &self.col_labels) {

            Ok(i) => i,
            Err(error) => return Err(error)

        };

        Ok(self.value_at_index(row_index, col_index))
    }

    pub fn add_col(&mut self, label: &'a str, content: Vec<Point<'a>>) {

        // Checks to make sure the dataframe is still valid
        assert_eq!(self.col_labels.len(), self.frame.len());

        self.col_labels.push(label);
        self.frame.push(content);

        assert_eq!(self.col_labels.len(), self.col_labels.len());
    }

    // Big confusion
    pub fn add_row(&mut self, label: &'a str, content: Vec<Point<'a>>) -> Result<(), String> {
        
        // Checks to make sure the dataframe is still valid
        assert_eq!(self.col_labels.len(), self.col_labels.len());

        // This was searching for an index of a label to delete instead of
        // adding the index to the end
        
        let _index: Option<usize> = match self.row_labels.iter().position(|x| *x == label) {

            Some(_i) => return Err(format!("This label already exists {:?}", label)),
            None => None
        };
        self.row_labels.push(label);
        for i in 0..self.frame.len() {
            self.frame[i].push(content[i].clone());
        }
        assert_eq!(self.row_labels.len(), self.row_labels.len());

        Ok(())
    }

    // TODO: Fix to switch rows and columns
    pub fn delete_row(&mut self, label: &'a str) -> Result<(), &str> {
        
        let index: usize = match self.row_labels.iter().position(|x| *x == label) {

            Some(i) => i,
            None => return Err("Row not found")
        };
        self.row_labels.remove(index);
        for i in 0..self.frame.len() {
            self.frame[i].remove(index);  
        }
        
        Ok(())
    }
    // TODO: Fix to switch columns and rows
    pub fn delete_column<'b>(&'b mut self, label: &'a str) -> Result<(), &'b str> {

        let index: usize = match self.col_labels.iter().position(|x| *x == label) {

            Some(i) => i,
            None => return Err("Column not found")
        };
        self.col_labels.remove(index);
        self.frame.remove(index);  
        
        Ok(())
    }

    pub fn label_index(label: &str, labels: &Vec<&str>) -> Result<usize, &'a str> {

        let mut col_num: Option<usize> = None;

        for i in 0..labels.len() {
            if labels[i] == label { 
                col_num = Some(i);
                break;
            }
        }

        match col_num {

            Some(n) => Ok(n),
            None => Err("This label doesn't exist")
        }
    }

    pub fn get_cols_len(&self) -> usize {

        self.col_labels.len()
    }

    pub fn get_rows_len(&self) -> usize {

        self.row_labels.len()
    }

    pub fn display(&self) {
        for _i in 0..self.row_labels[0].len() + 2 {
            print!(" ");
        }
        for i in &self.col_labels {
            print!("{:?} ", i)
        }
        print!("\n");
        for i in 0..self.row_labels.len() {
            print!("{:?} ", self.row_labels[i]);
            let longest = find_longest(&self.row_labels);
            let shortest = find_shortest(&self.row_labels);
            if longest != self.row_labels[i].len() {
                for _l in 0..longest - self.row_labels[0].len() {
                    print!(" ");
                }
            }
            for j in 0..self.col_labels.len() {
                self.frame[j][i].val.display();
                if longest != self.row_labels[j].len() && shortest != longest{
                    for _k in 0..self.col_labels[j].len() + 2 {
                        print!(" ");
                    }
                } else {
                    for _k in 0..self.col_labels[j].len() {
                        print!(" ");
                    }
                }
            }
            print!("\n");
        }
    }
}

impl<'a> Point<'a> {

    pub fn new(row: usize, col: usize, content: Types<'a>) -> Point<'a> {

        Point { row, col, val: content }
    }

    pub fn point_vector(col: usize, vector: Vec<Types<'a>>) -> Vec<Point<'a>> {

        let mut new_vector: Vec<Point<'a>> = vec![];

        for row in 0..vector.len() {
            new_vector.push(Point::new(row, col, vector[row]));
        }

        new_vector
    }
}

pub fn find_longest(vec: &Vec<&str>) -> usize {

    let mut longest_len: usize = vec[0].len();
    for i in 1..vec.len() {
        let length = vec[i].len();
        if length > longest_len { longest_len = length }
    }

    longest_len
}

pub fn find_shortest(vec: &Vec<&str>) -> usize {

    let mut shortest_len: usize = vec[0].len();
    for i in 1..vec.len() {
        let length = vec[i].len();
        if length < shortest_len { shortest_len = length; }
    }

    shortest_len
}