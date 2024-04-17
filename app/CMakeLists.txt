# This is a stub CMake file to sneakily divert to building with cargo

cmake_minimum_required(VERSION 3.10)
project(Compiledb)

add_library(compiledb SHARED stub.c)

add_custom_command(
  TARGET compiledb
  POST_BUILD
  COMMAND ${CMAKE_SOURCE_DIR}/build-lib-cargo.sh
  ARGS $<TARGET_FILE:compiledb> ${CMAKE_BUILD_TYPE} ${CMAKE_ANDROID_ARCH} ${CMAKE_C_COMPILER}
)
