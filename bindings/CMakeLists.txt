cmake_minimum_required(VERSION 3.14)

project(mcrl3_ffi
    VERSION 1.0.0
    DESCRIPTION "Foreign function interface for mcrl3"
    LANGUAGES CXX)
    
# Mark some cmake variables as advanced since they don't have to be exposed to the user.
mark_as_advanced(FORCE
    CORROSION_BUILD_TESTS
    CORROSION_DEV_MODE
    CORROSION_NATIVE_TOOLING
    CORROSION_NO_WARN_PARSE_TARGET_TRIPLE_FAILED
    CORROSION_RESPECT_OUTPUT_DIRECTORY
    CORROSION_VERBOSE_OUTPUT
    Rust_RESOLVE_RUSTUP_TOOLCHAINS)

# Use FetchContent to acquire Corrosion for the cmake integration of Rust projects.
include(FetchContent)

FetchContent_Declare(corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5.2)
FetchContent_MakeAvailable(corrosion)

find_program(CBINDGEN_EXECUTABLE
    NAMES cbindgen
    DOC "Requires cbindgen to generate the OxiDD headers"
    REQUIRED)

set(MCRL3_CARGO_PROFILE "release"
    CACHE STRING "Cargo profile to use for building the mcrl3-ffi crate")
set_property(CACHE MCRL3_CARGO_PROFILE PROPERTY STRINGS "dev" "release")

set(MCRL3_INCLUDE_DIR ${PROJECT_BINARY_DIR}/include)
set(MCRL3_INCLUDE_HEADER ${MCRL3_INCLUDE_DIR}/mcrl3_ffi.h)
set(MCRL3_ROOT_DIR "${PROJECT_SOURCE_DIR}/../")

add_custom_command(OUTPUT ${MCRL3_INCLUDE_HEADER}
    COMMAND ${CBINDGEN_EXECUTABLE}
    ARGS ${MCRL3_ROOT_DIR}/crates/aterm-ffi/ --output ${MCRL3_INCLUDE_HEADER}
    DEPENDS ${MCRL3_ROOT_DIR}/crates/aterm-ffi/cbindgen.toml
            ${MCRL3_ROOT_DIR}/crates/aterm-ffi/src/lib.rs
    COMMENT "Generating C header for the mcrl3_ffi crate"
    VERBATIM)
add_custom_target(mcrl3_ffi_header ALL DEPENDS ${MCRL3_INCLUDE_HEADER})
    
# Import the mcrl3_aterm-ffi crate specifically
corrosion_import_crate(
    MANIFEST_PATH ${MCRL3_ROOT_DIR}/Cargo.toml
    PROFILE ${MCRL3_CARGO_PROFILE}
    CRATES mcrl3_aterm-ffi
    CRATE_TYPES staticlib)

add_dependencies(mcrl3_aterm_ffi mcrl3_ffi_header)
target_include_directories(mcrl3_aterm_ffi INTERFACE
    ${MCRL3_INCLUDE_DIR}
    ${PROJECT_SOURCE_DIR}/include)

add_library(mcrl3_ffi ALIAS mcrl3_aterm_ffi)

add_subdirectory(example)