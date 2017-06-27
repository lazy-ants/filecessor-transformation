#[macro_use]
extern crate lazy_static;
extern crate opencv;
extern crate iron;
extern crate regex;

mod transformer;
use transformer::matchers;
use transformer::load;
use transformer::transformation;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;

use regex::*;

use std::fs::DirBuilder;

fn main() {
    fn handler(req: &mut Request) -> IronResult<Response> {
        let regex = Regex::new(r"^transform/([^\\/]+)/([^\\/]+)/(.+)").unwrap();
        match regex.captures(&req.url.path.join("/")) {
            Some(cap) => {
                let folder_param = cap.at(1).unwrap().to_string();
                let media_original = get_media_directory("MEDIA_ORIGINAL", &folder_param);
                let media_regular = get_media_directory("MEDIA_REGULAR", &folder_param);

                if folder_param != "-" {
                    let mut builder = DirBuilder::new();
                    builder
                        .recursive(true)
                        .create(["/media/" ,&folder_param, "/"].concat()).unwrap();
                }

                match matchers::create_operations(cap.at(2).unwrap()) {
                    Ok(operations) => match load::load_image(cap.at(3).unwrap(), &operations, &media_original, &media_regular) {
                        Ok(image) => {
                            let buffer = transformation::apply_operations(&image, &operations);
                            let content_type = load::ext_to_content_type(image.ext).parse::<Mime>().unwrap();
                            Ok(Response::with((content_type, status::Ok, buffer.to_vec())))
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
}

fn get_media_directory(key: &str, folder: &str) -> String {
    let value = std::env::var_os(key);

    if value.is_none() && folder == "-" {
        return "/media/".to_string();
    }

    if value.is_none() && folder != "-" {
        return ["/media/", folder, "/"].concat();
    }
    
    value.unwrap().into_string().unwrap()
}
