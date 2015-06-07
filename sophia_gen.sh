git clone git@github.com:crabtw/rust-bindgen.git
export LIBCLANG_PATH=/Library/Developer/CommandLineTools/usr/lib
cargo build

# clib install clibs/sophia
# clang -dynamiclib sophia.c -o libsophia.dylib
export DYLD_LIBRARY_PATH=`pwd`/deps/sophia

target/debug/bindgen deps/sophia/sophia.h -o deps/sophia/sophia.rs -l sophia