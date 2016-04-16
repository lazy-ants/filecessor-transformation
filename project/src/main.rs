extern crate opencv;
extern crate iron;

use opencv::core as cv;
use opencv::sys::types::{VectorOfint, VectorOfuchar};
use opencv::highgui;
use opencv::imgproc;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
		let mut buffer = VectorOfuchar::new();
		// let mut f = try!(File::open("/Users/dmitriybelyaev/Desktop/photo.jpg")
		// 	.map_err(|e| IronError::new(e, status::BadRequest)));

		// try!(f.read_to_end(&mut buffer)
		// 	.map_err(|e| IronError::new(e, status::BadRequest)));
		let mat = highgui::imread("/media/photo.jpg", highgui::IMREAD_COLOR).unwrap();
		
		// let mat = highgui::imread("/Users/dmitriybelyaev/Desktop/12672161_1042993885767412_1579124257236733045_o.jpg", highgui::IMREAD_COLOR).unwrap();
		let mut dest = cv::Mat::new().unwrap();
		let size = cv::Size { width: 400, height: 400 };

		imgproc::resize(&mat, &mut dest, size, 0.0, 0.0, imgproc::INTER_LINEAR);

		highgui::imencode(".jpg", &dest, &mut buffer, &VectorOfint::new());
        let content_type = "image/jpeg".parse::<Mime>().unwrap();
        Ok(Response::with((content_type, status::Ok, buffer.into_vec())))
    }

    Iron::new(hello_world).http("0.0.0.0:3000").unwrap();
    println!("On 3000");
}


// fn main() {
//     let mat = highgui::imread("/Users/dmitriybelyaev/Desktop/photo.jpg", highgui::IMREAD_COLOR).unwrap();
//     highgui::imwrite("/Users/dmitriybelyaev/Desktop/photo_handled.jpg", &mat, &VectorOfint::new());
// }
