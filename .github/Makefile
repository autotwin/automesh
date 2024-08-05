SHELL := /bin/bash

exodus:
	rm -fr seacas/ && \
	git clone --depth 1 https://github.com/sandialabs/seacas.git --single-branch --branch=master && \
	cd seacas/ && \
	git reset --hard 049a054ee0e1c1107336d5996136314f57ddbe2a && \
	export ACCESS=`pwd` && \
	cd TPL/hdf5/ && \
	wget https://github.com/HDFGroup/hdf5/releases/download/hdf5_1.14.4.3/hdf5-1.14.4-3.tar.gz && \
	tar fvxz hdf5-1.14.4-3.tar.gz && \
	cd hdf5-1.14.4-3/ && \
	bash ../runconfigure.sh && \
	make && \
	make install && \
	cd ../../netcdf/ && \
	wget https://downloads.unidata.ucar.edu/netcdf-c/4.9.2/netcdf-c-4.9.2.tar.gz && \
	tar fvxz netcdf-c-4.9.2.tar.gz && \
	cd netcdf-c-4.9.2/ && \
	mkdir build/ && \
	cd build/ && \
	bash ../../runcmake.sh && \
	cd ../../../../ && \
	./cmake-exodus
