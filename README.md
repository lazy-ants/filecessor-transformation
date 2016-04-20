# filecessor-transformation
Docker image to perform on-fly chains transformation for filecessor.com

# Transformation
To perform transformation on photo make request `/transform/{filter}/{filename}`
Filter can be list of transformation divided by `+` sign like that `{filter1}+{filter2}+...`. List is ordered (filters will be apply in such order it received).

# Filters:
 - Rotate (`rotate_{angle}`). Angle for rotation can be 90, 180, or 270 degrees
 - Resize (`resize_{width}x{height}`). Width and height can be size in pixels or one of them(not both) can be `-` sign for relative resize by one dimension(relative resize for width 500px will be `resize_500x-`)
 - Crop by coordinates (`crop_coordinates_{x1}x{y1}_{x2}x{y2}`). x1, y1 - top left point coordinates, x2, y2 - bootom right point coordinates
 - Crop by width and height (`crop_{width}x{height}`). Image will be cropped by width and height with central image alignment

# Example
photo.jpg should crop by 2 points(10x20), (1350, 1360), after that relative resized by height 300px, after that rotate by 180 degrees:
`/transform/crop_coordinates_10x20_1350x1360+resize_-x300+rotate_180/photo.jpg`
 
