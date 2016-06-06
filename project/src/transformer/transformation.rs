extern crate opencv;

use opencv::core as cv;
use opencv::types::{VectorOfint, VectorOfuchar};
use opencv::highgui;
use opencv::imgproc;

#[derive(Debug)]
pub enum Transformation {
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

pub struct Image {
    pub ext: String,
    pub mat: cv::Mat
}

pub fn apply_operations(image: &Image, operations: &Vec<Transformation>) -> VectorOfuchar {
    let mut mat = operations[0].apply(&image.mat);

    let mut i = 1;
    while i < operations.len() {
        mat = operations[i].apply(&mat);
        i += 1;
    }

    let mut buffer = VectorOfuchar::new();
    highgui::imencode(&format!(".{}", image.ext), &mat, &mut buffer, &VectorOfint::new());

    buffer
}

impl Transformation {
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
                let original_size = mat.size().unwrap();
                let original_proportions = (original_size.width as f32) / (original_size.height as f32);
                let crop_proportions = width as f32 / height as f32;

                let rect: cv::Rect;
                let resized: cv::Mat;
                if crop_proportions > original_proportions {
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
    imgproc::resize(&mat, &mut dest, *size, 0.0, 0.0, imgproc::INTER_AREA);

    dest
}
