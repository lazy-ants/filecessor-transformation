#[macro_use]
extern crate lazy_static;
extern crate opencv;
extern crate iron;
extern crate regex;
extern crate rexiv2;

use opencv::core as cv;
use opencv::types::{VectorOfint, VectorOfuchar};
use opencv::highgui;
use opencv::imgproc;

use std::io::prelude::*;
use std::option::*;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;

use regex::*;
use std::path::Path;

lazy_static! {
    static ref MEDIA_DIRECTORY: String = get_media_directory();
}

fn main() {
    fn handler(req: &mut Request) -> IronResult<Response> { 
        let regex = Regex::new(r"^transform/(.+)/(.+)\.(?i)(jpg|jpeg|png|tif|tiff)$").unwrap();
        match regex.captures(&req.url.path.join("/")) {
            Some(cap) => {
                let ext = cap.at(3).unwrap();
                let path_regular = format!("{}regular/{}.{}", MEDIA_DIRECTORY.to_string(), cap.at(2).unwrap(), ext);
                let path_original = format!("{}original/{}.{}", MEDIA_DIRECTORY.to_string(), cap.at(2).unwrap(), ext);

                if !Path::new(&path_original).exists() {
                    return Ok(Response::with((iron::status::NotFound, "Image not found")));
                }

                return handle_image(cap.at(1).unwrap(), &path_original, &path_regular, ext);
            },
            None => Ok(Response::with((iron::status::NotFound, "Invalid url"))),
        }
    }
    
    Iron::new(handler).http("0.0.0.0:3000").unwrap();
}

fn handle_image(filters: &str, path_original: &str, path_regular: &str, ext: &str) -> IronResult<Response> {
    let splitted: Vec<&str> = filters.split("+").collect();
    let mut operations: Vec<Transformation> = Vec::new();
    for entry in &splitted {
        match create_operation(entry) {
            Some(operation) => {
                operations.push(operation);
            },
            None => {
                return Ok(Response::with((iron::status::BadRequest,  format!("Invalid transformation step \"{}\"", entry))));
            }
        }
    }

    let path: &str;
    if is_regular_can_be_used(&operations, path_regular) {
        path = path_regular;
    } else {
        path = path_original;
    }

    let mut buffer = VectorOfuchar::new();
    let mut mat = highgui::imread(path, highgui::IMREAD_UNCHANGED).unwrap();

    for operation in &operations {
        mat = operation.apply(&mat);
    }

    highgui::imencode(&format!(".{}", ext), &mat, &mut buffer, &VectorOfint::new());
    
    let content_type = match String::from(ext).to_lowercase().as_ref() {
        "jpg"|"jpeg" => "image/jpeg",
        "png" => "image/png",
        "tif"|"tiff" => "image/tiff",
        _ => "text/plain"
    }.parse::<Mime>().unwrap();

    Ok(Response::with((content_type, status::Ok, buffer.to_vec())))
}

fn is_regular_can_be_used(operations: &Vec<Transformation>, path_regular: &str) -> bool {
    if Path::new(&path_regular).exists() {
        return match rexiv2::Metadata::new_from_path(path_regular) {
            Ok(meta) => {
                let width = meta.get_pixel_width();
                let height = meta.get_pixel_height();
                let mut result = false;

                for operation in operations {
                    match operation.is_regular_can_be_used(width, height) {
                        Cache::NOT_AFFECT => {
                            continue;
                        },
                        Cache::NOT_APPLIED => {
                            result = false;
                            break;
                        },
                        Cache::APPLIED => {
                            result = true;
                            break;
                        }
                    }
                }

                result
            },
            Err(err) => false
        }
    }

    return false;
}

#[derive(Debug)]
enum Transformation {
    Resize {
        height: Option<i32>,
        width: Option<i32>
    },
    Crop {
        height: i32,
        width: i32
    },
    CropCoordinates {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32
    },
    Rotate {
        degrees: i32
    }
}

enum Cache {
    APPLIED,     // return if image can be based on regular
    NOT_APPLIED, // cannot be based on regular
    NOT_AFFECT   // has no affect for image size
}

trait TransformationTrait {
    fn apply(&self, mat: &cv::Mat) -> cv::Mat;

    // Return true if regular image can be read for this transformation
    fn is_regular_can_be_used(&self, width: i32, height: i32) -> Cache;
}

impl TransformationTrait for Transformation {
    fn apply(&self, mat: &cv::Mat) -> cv::Mat {
        match *self {
            Transformation::Resize { height, width } => {
                let size: cv::Size;
                if width.is_some() && height.is_some() {
                    size = cv::Size { width: width.unwrap(), height: height.unwrap() };
                    return resize(&mat, &size);
                } else if width.is_some() {
                    return relative_resize_width(&mat, width.unwrap());
                } else {
                    return relative_resize_height(&mat, height.unwrap());
                }
            },
            Transformation::Rotate { degrees } => {
                let mut dest = cv::Mat::new().unwrap();
                let mut final_dest = cv::Mat::new().unwrap();
                
                match degrees {
                    90 => {
                        cv::transpose(&mat, &mut dest);
                        cv::flip(&dest, &mut final_dest, 1);

                        final_dest
                    },
                    180 => {
                        cv::flip(&mat, &mut final_dest, -1);

                        final_dest
                    },
                    270 => {
                        cv::transpose(&mat, &mut dest);
                        cv::flip(&dest, &mut final_dest, 0);

                        final_dest
                    },
                    _ => dest
                }
            },
            Transformation::Crop { height, width } => {
                let rect: cv::Rect;
                let resized: cv::Mat;
                if width > height {
                    resized = relative_resize_width(&mat, width);
                    rect = cv::Rect {
                        x: 0,
                        y: (height - resized.size().unwrap().height).abs() / 2,
                        width: width,
                        height: height
                    };
                } else {
                    resized = relative_resize_height(&mat, height);
                    rect = cv::Rect {
                        x: (width - resized.size().unwrap().width).abs() / 2,
                        y: 0,
                        width: width,
                        height: height
                    };
                }
                return cv::Mat::rect(&resized, rect).unwrap();
            },
            Transformation::CropCoordinates { x1, y1, x2, y2 } => {
                let rect = cv::Rect {
                    x: x1,
                    y: y1,
                    width: (x2 - x1).abs(),
                    height: (y2 - y1).abs()
                };
                return cv::Mat::rect(&mat, rect).unwrap();
            }
        }
    }
    fn is_regular_can_be_used(&self, regular_width: i32, regular_height: i32) -> Cache {
        match *self {
            Transformation::Resize { height, width } => {
                let condition: bool;
                if height.is_some() && width.is_some() {
                    condition = regular_width >= width.unwrap() && regular_height >= height.unwrap();
                } else if height.is_some() {
                    condition = regular_height >= height.unwrap();
                } else {
                    condition = regular_width >= width.unwrap();
                }

                if condition {
                    return Cache::APPLIED;
                } else {
                    return Cache::NOT_APPLIED;
                }
            },
            Transformation::CropCoordinates { x1, y1, x2, y2 } => Cache::NOT_APPLIED,
            Transformation::Crop { height, width } => {
                if regular_width >= width && regular_height >= height {
                    return Cache::APPLIED;
                } else {
                    return Cache::NOT_APPLIED;
                }
            },
            Transformation::Rotate { degrees } => Cache::NOT_AFFECT
        }
    }
}

fn create_operation(entry: &str) -> Option<Transformation> {
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

fn relative_resize_width(mat: &cv::Mat, width: i32) -> cv::Mat {
    let given_size = mat.size().unwrap();
    let height = width * given_size.height / given_size.width;
    let size = cv::Size { width: width, height: height };

    resize(mat, &size)
}

fn relative_resize_height(mat: &cv::Mat, height: i32) -> cv::Mat {
    let given_size = mat.size().unwrap();
    let width = height * given_size.width / given_size.height;
    let size = cv::Size { width: width, height: height };

    resize(mat, &size)
}

fn resize(mat: &cv::Mat, size: &cv::Size) -> cv::Mat {
    let mut dest = cv::Mat::new().unwrap();
    imgproc::resize(&mat, &mut dest, *size, 0.0, 0.0, imgproc::INTER_LINEAR);

    dest
}

fn get_media_directory() -> String {
    let key = "MEDIA_DIRECTORY";
    let value = std::env::var_os(key);
    if value.is_none() {
        return "/media/".to_string();
    }
    
    value.unwrap().into_string().unwrap()
}
