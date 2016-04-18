FROM ubuntu:14.04

RUN apt-get update && apt-get -y install curl cmake g++ unzip python pkg-config

RUN mkdir -p /tmp/opencv && cd /tmp/opencv && curl -O http://iweb.dl.sourceforge.net/project/libjpeg-turbo/1.4.2/libjpeg-turbo-official_1.4.2_amd64.deb && \
    dpkg -i libjpeg-turbo-official_1.4.2_amd64.deb

RUN mkdir -p /tmp/opencv && cd /tmp/opencv && curl -O https://codeload.github.com/Itseez/opencv/zip/2.4.12 && \
    unzip 2.4.12 && mkdir opencv-2.4.12/build && cd opencv-2.4.12/build && \
    cmake -D CMAKE_BUILD_TYPE=RELEASE -D CMAKE_INSTALL_PREFIX=/usr/local -DWITH_JPEG=ON -DBUILD_JPEG=OFF -DJPEG_INCLUDE_DIR=/opt/libjpeg-turbo/include/ -DJPEG_LIBRARY=/opt/libjpeg-turbo/lib64/libjpeg.a .. && \
    make && make install && pkg-config --cflags opencv && pkg-config --libs opencv && rm -rf /tmp/opencv

RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh

COPY ./project /tmp/app

RUN cd /tmp/app && cargo build --release && mv target/release/transformer /usr/bin && rm -rf /tmp/app

ENV LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/local/lib
ENV MEDIA_DIRECTORY="/media/"

EXPOSE 3000

VOLUME ["/media"]

CMD ["transformer"]
