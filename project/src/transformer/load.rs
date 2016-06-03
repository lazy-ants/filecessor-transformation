extern crate rexiv2;
extern crate regex;
extern crate opencv;
extern crate hyper;

use super::transformation::*;
use self::regex::*;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::hash::*;

use self::hyper::Client;
use self::hyper::header::Connection;

use opencv::core::*;
use opencv::highgui;

pub fn load_image(relative_path: &str, operations: &Vec<Transformation>, original: &str, regular: &str) -> Result<Image, String> {
    println!("{:?}", relative_path);
    let url_regex = Regex::new(r"^https?://([\da-z.-]+)").unwrap();
    match url_regex.captures(relative_path) {
        Some(cap) => load_image_by_url(original, relative_path),
        None => {
            let regex = Regex::new(r"(.+)\.(?i)(jpe?g|png|tiff?)$").unwrap();
            
            match regex.captures(relative_path) {
                Some(cap) => load_image_from_disk(relative_path, operations, original, regular),
                None => Err("Invalid format".to_string())
            }
        }
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

pub fn content_type_to_ext(content_type: String) -> String {
    return match content_type.as_ref() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/tiff" => "tif",
        _ => ""
    }.to_string();
}

enum Cache {
    APPLIED,     // return if image can be based on regular
    NOT_APPLIED, // cannot be based on regular
    NOT_AFFECT   // has no affect for image size
}

fn load_image_by_url(original: &str, url: &str) -> Result<Image, String> {
    let string_path = format!("{}{}.jpg", original.to_string(), hash_string(url.to_string()));
    println!("{:?}", string_path);
    let client = Client::new();
    let mut res = client.get(url)
        .header(Connection::close())
        .send().unwrap();

    if (res.status.is_success()) {
        println!("Headers:\n{}", res.headers);

        let mut buffer = Vec::new();
        res.read_to_end(&mut buffer).unwrap();
        let path = Path::new(&string_path);

        return match File::create(&path) {
            Err(_) => Err("System Error".to_string()),
            Ok(mut file) => match file.write_all(&buffer) {
                Err(_) => Err("System Error".to_string()),
                Ok(_) => Ok(Image {
                    ext: "jpg".to_string(),
                    mat: highgui::imread(&string_path, highgui::IMREAD_UNCHANGED).unwrap()
                }),
            }
        };
    }
    
    Err("Invalid image link".to_string())
}

fn load_image_from_disk(relative_path: &str, operations: &Vec<Transformation>, original: &str, regular: &str) -> Result<Image, String> {
    let path_regular = format!("{}{}", regular.to_string(), relative_path);
    let path_original = format!("{}{}", original.to_string(), relative_path);

    if !Path::new(&path_original).exists() {
        return Err("Image not found".to_string());
    }
    
    //read metadata from regular or from original
    let (meta, using_regular) = match rexiv2::Metadata::new_from_path(&path_regular) {
        Ok(meta) => (Ok(meta), true),
        Err(_) => (rexiv2::Metadata::new_from_path(&path_original), false),
    };

    return match meta {
        Ok(meta) => {
            let using_path = if using_regular && check_image_for_using_regular(meta.get_pixel_width(), meta.get_pixel_height(), operations) {
                &path_regular
            } else {
                &path_original
            };
            
            let ext = content_type_to_ext(meta.get_media_type().unwrap());

            create_image(using_path, &ext)
        },
        Err(_) => Err("Crashed image".to_string()),
    }
}

fn check_image_for_using_regular(width: i32, height: i32, operations: &Vec<Transformation>) -> bool {
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

fn create_image(path: &str, ext: &str) -> Result<Image, String> {
    match highgui::imread(path, highgui::IMREAD_UNCHANGED) {
        Ok(mat) => Ok(Image {
            ext: ext.to_string(),
            mat: mat
        }),
        Err(_) => Err("Crashed image".to_string()),
    } 
}

fn hash_string(obj: String) -> u64 {
    let mut hasher = SipHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}
