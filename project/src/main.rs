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

lazy_static! {
    static ref MEDIA_ORIGINAL: String = get_media_directory("MEDIA_ORIGINAL");
    static ref MEDIA_REGULAR: String = get_media_directory("MEDIA_REGULAR");
}

fn main() {
    fn handler(req: &mut Request) -> IronResult<Response> { 
        let regex = Regex::new(r"^transform/(.+)/(.+)").unwrap();
        match regex.captures(&req.url.path.join("/")) {
            Some(cap) => {
                match matchers::create_operations(cap.at(1).unwrap()) {
                    Ok(operations) => match load::load_image(cap.at(2).unwrap(), &operations, &MEDIA_ORIGINAL, &MEDIA_REGULAR) {
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

fn get_media_directory(key: &str) -> String {
    let value = std::env::var_os(key);
    if value.is_none() {
        return "/media/".to_string();
    }
    
    value.unwrap().into_string().unwrap()
}
