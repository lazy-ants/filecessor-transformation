extern crate opencv;
extern crate iron;
extern crate regex;

use opencv::core as cv;
use opencv::sys::types::{VectorOfint, VectorOfuchar};
use opencv::highgui;
use opencv::imgproc;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::option::*;
use std::fmt;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;

use regex::*;
use std::path::Path;

fn main() {
    fn handler(req: &mut Request) -> IronResult<Response> {        
  		let directory = "/Users/dmitriybelyaev/Development/rust/transformer/media/";
  		let regex = Regex::new(r"^transform/(.+)/(.+)\.([a-zA-Z]{3, 4})$").unwrap();
  		match regex.captures(&req.url.path.join("/")) {
  		    Some(cap) => {
  		    	let ext = cap.at(3).unwrap();
  		    	let path = format!("{}{}.{}", directory.to_string(), cap.at(2).unwrap(), ext);

  		    	if !Path::new(&path).exists() {
  		    		return Ok(Response::with((iron::status::NotFound, "Image not found")));
  		    	}

  		    	return handle_image(cap.at(1).unwrap(), &path, ext);
  		    },
  		    None => Ok(Response::with((iron::status::NotFound, "Invalid url"))),
  		}
    }

    Iron::new(handler).http("0.0.0.0:3000").unwrap();
    println!("On 3000");
}

fn handle_image(filters: &str, path: &str, ext: &str) -> IronResult<Response> {
	let splitted: Vec<&str> = filters.split("+").collect();
    let mut operations: Vec<Transformation> = Vec::new();
    for entry in &splitted {
        match create_operation(entry) {
            Some(operation) => {
                operations.push(operation);
            },
            None => {
                return Ok(Response::with((iron::status::BadRequest,  format!("Invalid transformation url \"{}\"", entry))));
            }
        }
    }

    let mut buffer = VectorOfuchar::new();
	let mat = highgui::imread(path, highgui::IMREAD_COLOR).unwrap();
	
	let mut dest = cv::Mat::new().unwrap();

	println!("{:?}", operations);
	for operation in &operations {
	    dest = operation.apply(&mat);
	}

	highgui::imencode(".jpg", &dest, &mut buffer, &VectorOfint::new());
    let content_type = "image/jpeg".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, buffer.into_vec())))
}

#[derive(Debug)]
enum Transformation {
    Resize {
    	height: Option<i32>,
    	width: Option<i32>
    },
    Rotate {
    	degrees: i32
    }
}

trait TransformationTrait {
    fn apply(&self, mat: &cv::Mat) -> cv::Mat;
}

impl TransformationTrait for Transformation {
    fn apply(&self, mat: &cv::Mat) -> cv::Mat {
    	match *self {
    	    Transformation::Resize { height: height, width: width } => {
    	    	let size = cv::Size { width: width.unwrap(), height: height.unwrap() };
    	    	let mut dest = cv::Mat::new().unwrap();

				imgproc::resize(&mat, &mut dest, size, 0.0, 0.0, imgproc::INTER_LINEAR);

				dest
    	    },
    	    Transformation::Rotate { degrees: degrees } => {
				let size = cv::Size { width: 200, height: 200 };
    	    	let mut dest = cv::Mat::new().unwrap();

				imgproc::resize(&mat, &mut dest, size, 0.0, 0.0, imgproc::INTER_LINEAR);

				dest
    	    },
    	}
    }
}

fn create_operation(entry: &str) -> Option<Transformation> {
    let matchers:Vec<fn(&str) -> Option<Transformation>> = vec![match_resize, match_rotate];
    
    for matcher in &matchers {
        let option = matcher(entry);
        if (option.is_some()) {
            return option; 
        }
    }

    None
}

fn match_resize(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"resize_(\d+)x(\d+)").unwrap();

    regex.captures(entry).and_then(|cap: Captures| Option::Some(Transformation::Resize {
        width: Some(cap.at(1).unwrap().parse::<i32>().unwrap()),
        height: Some(cap.at(2).unwrap().parse::<i32>().unwrap())
    }))
}

fn match_rotate(entry: &str) -> Option<Transformation> {
    let regex = Regex::new(r"rotate_(\d+)").unwrap();

    regex.captures(entry).and_then(|cap: Captures| Option::Some(Transformation::Rotate {
        degrees: cap.at(1).unwrap().parse::<i32>().unwrap(),
    }))
}
