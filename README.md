# Filecessor Transformation
The part of Filercessor system, is the docker image (container) used to perform 
on-fly chains transformations on images

## Transformation

- To perform transformation on photo make request `/transform/{filter}/{filename_or_url}`
- Filter can be list of transformation divided by `+` sign like that `{filter1}+{filter2}+...`. 
- List is ordered (filters will be apply in such order it received).

## Available Filters:

 - **Rotate** (`rotate_{angle}`). 
  - Rotation angles is 90, 180, or 270 degrees
 - **Resize** (`resize_{width}x{height}`)
  - Width and height can be size in pixels or one of them (not both) can be `-` sign for relative resize by one dimension (relative resize for width 500px will be `resize_500x-`)
 - **Crop by coordinates** (`crop_coordinates_{x1}x{y1}_{x2}x{y2}`)
  - x1, y1 - top left point coordinates, x2, y2 - bootom right point coordinates
 - **Crop** by width and height (`crop_{width}x{height}`)
  - note: image cropped by width and height with central image alignment

## Supported formats

- jpg
- jpeg
- png
- tif
- tiff

## Geting start your own instance with Docker

- install [Docker](http://docker.com) and [Docker Compose](https://docs.docker.com/compose/)
- `git clone` this repo to `somewhere/transformation`
- run containers with `docker-composer up -d`
- open in browser `http://localhost/transform/resize_400x-+rotate_90/http://dreamatico.com/data_images/babies/babies-1.jpg`

## Example

we have an image `photo.jpg` with below transformation chain:
- crop by 2 points (10x20) and (1350, 1360)
- relatively resize by height 300px
- rotate to 180 degrees

`http://filecessor.com/transform/crop_coordinates_10x20_1350x1360+resize_-x300+rotate_180/photo.jpg`

Transform Photo from by Url:

`http://filecessor.com/transform/crop_coordinates_10x20_1350x1360+resize_-x300+rotate_180/http://external_resource.com/photo.jpg`

## Api

- Read API documentation about how to use the service http://docs.filecessor.apiary.io

## Contributing

Filecessor is an open source project. If you find bugs or have proposal please create [issue](https://github.com/lazy-ants/filecessor/issues) or Pull Request
    
## License

All what you can find at this repo shared under the MIT License

 
