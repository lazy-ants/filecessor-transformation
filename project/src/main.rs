#[macro_use]
extern crate lazy_static;
extern crate opencv;
extern crate iron;
extern crate regex;
extern crate rexiv2;

mod transformer;

use transformer::matchers;
use transformer::load;

// use opencv::core as cv;
// use opencv::types::{VectorOfint, VectorOfuchar};
use opencv::highgui;
// use opencv::imgproc;

// use std::io::prelude::*;
// use std::option::*;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;

use regex::*;
// use std::path::Path;

lazy_static! {
    static ref MEDIA_ORIGINAL: String = get_media_directory("MEDIA_ORIGINAL");
    static ref MEDIA_REGULAR: String = get_media_directory("MEDIA_REGULAR");
}

fn main() {
    fn handler(req: &mut Request) -> IronResult<Response> { 
        let regex = Regex::new(r"^transform/(.+)/(.+)").unwrap();
        match regex.captures(&req.url.path.join("/")) {
            Some(cap) => {

                // let ext = cap.at(3).unwrap();
                // let path_regular = format!("{}regular/{}.{}", MEDIA_DIRECTORY.to_string(), cap.at(2).unwrap(), ext);
                // let path_original = format!("{}original/{}.{}", MEDIA_DIRECTORY.to_string(), cap.at(2).unwrap(), ext);

                // if !Path::new(&path_original).exists() {
                //     return Ok(Response::with((iron::status::NotFound, "Image not found")));
                // }

                // return handle_image(cap.at(1).unwrap(), &path_original, &path_regular, ext);
                match matchers::create_operations(cap.at(1).unwrap()) {
                    Ok(operations) => match load::load_image(cap.at(2).unwrap(), &operations, &MEDIA_ORIGINAL, &MEDIA_REGULAR) {
                        Ok(mat) => {
                            println!("{:?}", mat.ext);
                            Ok(Response::with((iron::status::Ok, "ok")))
                        },
                        Err(message) => Ok(Response::with((iron::status::NotFound, message)))
                    },
                    Err(message) => Ok(Response::with((iron::status::BadRequest, message)))
                }

                
            },
            None => Ok(Response::with((iron::status::NotFound, "Invalid url"))),
        }
    }
    
    Iron::new(handler).http("0.0.0.0:3000").unwrap();
    // println!("Hello");
}

// fn handle_image(filters: &str, path_original: &str, path_regular: &str, ext: &str) -> IronResult<Response> {
//     let splitted: Vec<&str> = filters.split("+").collect();
//     let mut operations: Vec<Transformation> = Vec::new();
//     for entry in &splitted {
//         match create_operation(entry) {
//             Some(operation) => {
//                 operations.push(operation);
//             },
//             None => {
//                 return Ok(Response::with((iron::status::BadRequest,  format!("Invalid transformation step \"{}\"", entry))));
//             }
//         }
//     }

//     let path: &str;
//     if is_regular_can_be_used(&operations, path_regular) {
//         path = path_regular;
//     } else {
//         path = path_original;
//     }

//     let mut buffer = VectorOfuchar::new();
//     let mut mat = highgui::imread(path, highgui::IMREAD_UNCHANGED).unwrap();

//     for operation in &operations {
//         mat = operation.apply(&mat);
//     }

//     highgui::imencode(&format!(".{}", ext), &mat, &mut buffer, &VectorOfint::new());
    
    // let content_type = match String::from(ext).to_lowercase().as_ref() {
    //     "jpg"|"jpeg" => "image/jpeg",
    //     "png" => "image/png",
    //     "tif"|"tiff" => "image/tiff",
    //     _ => "text/plain"
    // }.parse::<Mime>().unwrap();

//     Ok(Response::with((content_type, status::Ok, buffer.to_vec())))
// }

fn get_media_directory(key: &str) -> String {
    let value = std::env::var_os(key);
    if value.is_none() {
        return "/media/".to_string();
    }
    
    value.unwrap().into_string().unwrap()
}
