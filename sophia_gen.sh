#!/bin/bash

function compile_sophia {
    cd sophia
    make sophia.h sophia.c
    # clang -dynamiclib sophia.c -o libsophia.dylib
    # export DYLD_LIBRARY_PATH=`pwd`/deps/sophia
    # target/debug/bindgen deps/sophia/sophia.h -o deps/sophia/sophia.rs -l sophia
}

function generate_bindings {
    git clone git@github.com:crabtw/rust-bindgen.git
    export LIBCLANG_PATH=/Library/Developer/CommandLineTools/usr/lib
    cd rust-bindgen
    cargo build
}

compile_sophia
