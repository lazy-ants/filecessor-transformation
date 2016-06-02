extern crate rexiv2;
extern crate regex;
extern crate opencv;

use super::transformation::*;
use self::regex::*;
use std::path::Path;


use opencv::core::*;
use opencv::highgui;

pub fn load_image(relative_path: &str, operations: &Vec<Transformation>, original: &str, regular: &str) -> Result<Image, String> {
    let regex = Regex::new(r"(.+)\.(?i)(jpe?g|png|tiff?)$").unwrap();
    match regex.captures(relative_path) {
        Some(cap) => {
            let path_regular = format!("{}{}", regular.to_string(), relative_path);
            let path_original = format!("{}{}", original.to_string(), relative_path);

            if !Path::new(&path_original).exists() {
                return Err("Image not found".to_string());
            }
            
            let using_path: &str;
            if (check_image_for_using_regular(path_regular.as_ref(), operations)) {
                using_path = &path_regular;
            } else {
                using_path = &path_original;
            }
            
            return match highgui::imread(using_path, highgui::IMREAD_UNCHANGED) {
                Ok(mat) => Ok(Image {
                    ext: cap.at(2).unwrap().to_string(),
                    mat: mat
                }),
                Err(err) => Err("Crashed image".to_string()),
            }
        },
        None => Err("Invalid format".to_string())
    }
}

pub fn ext_to_content_type(ext: String) -> String {
    return match ext.to_lowercase().as_ref() {
        "jpg"|"jpeg" => "image/jpeg",
        "png" => "image/png",
        "tif"|"tiff" => "image/tiff",
        _ => "text/plain"
    }.to_string();
}

enum Cache {
    APPLIED,     // return if image can be based on regular
    NOT_APPLIED, // cannot be based on regular
    NOT_AFFECT   // has no affect for image size
}

fn check_image_for_using_regular(path_regular: &str, operations: &Vec<Transformation>) -> bool {
    if Path::new(&path_regular).exists() {
        return match rexiv2::Metadata::new_from_path(path_regular) {
            Ok(meta) => {
                let width = meta.get_pixel_width();
                let height = meta.get_pixel_height();
                let mut result = false;

                for operation in operations {
                    match check_operation_for_using_regular(&operation, width, height) {
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
            Err(_) => false
        }
    }

    return false;
}

fn check_operation_for_using_regular(operation: &Transformation, regular_width: i32, regular_height: i32) -> Cache {
    match *operation {
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
