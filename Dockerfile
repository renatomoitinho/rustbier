FROM rust:latest

RUN apt-get update

RUN apt-get install -y software-properties-common

RUN add-apt-repository "deb http://security.ubuntu.com/ubuntu xenial-security main"
RUN apt-get update

#opencv deps
RUN apt-get -y install build-essential cmake git libgtk2.0-dev libjpeg62-turbo pkg-config libavcodec-dev libavformat-dev libswscale-dev python-dev python-numpy libtbb2 libtbb-dev libjpeg-dev libpng-dev libdc1394-22-dev locales clang libclang1

RUN wget https://github.com/opencv/opencv/archive/3.4.6.zip
RUN unzip 3.4.6

RUN wget https://www.imagemagick.org/download/ImageMagick.tar.gz
RUN mkdir imagemagick
RUN tar xvzf ImageMagick.tar.gz -C imagemagick --strip-components 1

# install opencv
RUN mkdir opencv-3.4.6/build
RUN cd opencv-3.4.6/build && cmake  -DCMAKE_BUILD_TYPE=Release -DBUILD_PERF_TESTS=OFF -DBUILD_TESTS=OFF -DINSTALL_TESTS=OFF -DOPENCV_GENERATE_PKGCONFIG=ON -DBUILD_DOCS=OFF -DBUILD_EXAMPLES=OFF -DBUILD_opencv_apps=ALL -DWITH_IPP=OFF -DPYTHON_EXECUTABLE=OFF -DINSTALL_PYTHON_EXAMPLES=OFF -DWITH_LAPACK=ON -DWITH_EIGEN=ON -DBUILD_SHARED_LIBS=ON -DWITH_TBB=ON -DOPENCV_ENABLE_NONFREE=ON -DCMAKE_INSTALL_PREFIX=/usr/local ..
RUN cd opencv-3.4.6/build && make -j7 
RUN cd opencv-3.4.6/build && make install

# install imagemagick
RUN cd imagemagick && ./configure
RUN cd imagemagick && make
RUN cd imagemagick && make install
RUN cd imagemagick && ldconfig /usr/local/lib

# Set the locale
RUN locale-gen en_US.UTF-8
RUN localedef -i en_US -f UTF-8 en_US.UTF-8
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && locale-gen
ENV LANG en_US.UTF-8  
ENV LANGUAGE en_US:en  
ENV LC_ALL en_US.UTF-8 
ENV LD_LIBRARY_PATH /usr/local/lib

WORKDIR /usr/src/rustbier
COPY . .

RUN cargo install --path .

WORKDIR /

RUN rm -rf /usr/src

CMD rustbier