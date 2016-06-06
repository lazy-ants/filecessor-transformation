extern crate regex;

use super::transformation::Transformation;
use self::regex::*;

pub fn create_operations(entry: &str) -> Result<Vec<Transformation>, String> {
    let splitted: Vec<&str> = entry.split("+").collect();
    let mut operations: Vec<Transformation> = Vec::new();
    for entry in &splitted {
        match create_operation(entry) {
            Some(operation) => {
                operations.push(operation);
            },
            None => {
                return Err(format!("Invalid transformation \"{}\"", entry));
            }
        }
    }

    Ok(operations)
}

pub fn create_operation(entry: &str) -> Option<Transformation> {
    let matchers:Vec<fn(&str) -> Option<Transformation>> = vec![match_resize, match_rotate, match_crop, match_crop_coordinates];
    
    for matcher in &matchers {
        let option = matcher(entry);
        if option.is_some() {
            return option; 
        }
    }

    None
}

fn match_resize(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"resize_(\d+|-)x(\d+|-)").unwrap();
    
    regex.captures(entry).and_then(|cap| {
        let width = cap.at(1).unwrap();
        let height = cap.at(2).unwrap();
        if width == "-" && height == "-" {
            return None;
        }

        Option::Some(Transformation::Resize {
            width: if width == "-" {
                None 
            } else  {
                Some(width.parse::<i32>().unwrap())
            },
            height: if height == "-" {
                None
            } else { 
                Some(height.parse::<i32>().unwrap())
            },
        })
    })
}

fn match_rotate(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"rotate_(\d+)").unwrap();

    regex.captures(entry).and_then(|cap| {
        let degrees = cap.at(1).unwrap().parse::<i32>().unwrap();
        if degrees != 90 && degrees != 180 && degrees != 270 {
            return None;
        }

        Option::Some(Transformation::Rotate {
            degrees: degrees,
        })   
    })
}

fn match_crop(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"crop_(\d+)x(\d+)").unwrap();

    regex.captures(entry).and_then(|cap| Option::Some(Transformation::Crop {
        width: cap.at(1).unwrap().parse::<i32>().unwrap(),
        height: cap.at(2).unwrap().parse::<i32>().unwrap()
    }))
}

fn match_crop_coordinates(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"crop_coordinates_(\d+)x(\d+)_(\d+)x(\d+)").unwrap();

    regex.captures(entry).and_then(|cap| Option::Some(Transformation::CropCoordinates {
        x1: cap.at(1).unwrap().parse::<i32>().unwrap(),
        y1: cap.at(2).unwrap().parse::<i32>().unwrap(),
        x2: cap.at(3).unwrap().parse::<i32>().unwrap(),
        y2: cap.at(4).unwrap().parse::<i32>().unwrap(),
    }))
}
